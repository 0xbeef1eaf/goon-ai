use crate::assets::registry::AssetRegistry;
use crate::assets::selector::AssetSelector;
use crate::assets::types::Asset;
use crate::config::pack::Mood;
use crate::gui::window_manager::GuiInterface;
use crate::media::video::content::VideoContent;
use crate::media::video::downloader::MpvDownloader;
use crate::media::video::player::VideoPlayer;
use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
use crate::sdk::types::WindowOptions;
use deno_core::OpState;
use deno_core::op2;
use serde::Deserialize;
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use ts_rs::TS;

#[derive(Deserialize, Debug, Default, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct VideoOptions {
    pub tags: Option<Vec<String>>,
    pub loop_: Option<bool>, // "loop" is a keyword
    pub volume: Option<f32>,
    pub autoplay: Option<bool>,
    pub duration: Option<u64>,
    #[serde(flatten)]
    pub window: WindowOptions,
}

#[op2(async)]
#[string]
pub async fn op_show_video(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<String, OpError> {
    let (gui_controller, registry, mood) = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "video")?;
        let gui = state.borrow::<Arc<dyn GuiInterface>>().clone();
        let registry = state.borrow::<Arc<AssetRegistry>>().clone();
        let mood = state.borrow::<Mood>().clone();
        (gui, registry, mood)
    };

    let opts: VideoOptions = if let Some(o) = options {
        serde_json::from_value(o).map_err(|e| OpError::new(&e.to_string()))?
    } else {
        VideoOptions::default()
    };

    let tags = opts.tags.clone().unwrap_or_default();
    let selector = AssetSelector::new(&registry);

    let asset = selector
        .select_video(&mood, &tags)
        .ok_or_else(|| OpError::new("No video found matching tags"))?;

    let path = match asset {
        Asset::Video(vid) => &vid.path,
        _ => return Err(OpError::new("Selected asset is not a video")),
    };

    println!("Showing video: {:?} with options: {:?}", path, opts);

    MpvDownloader::ensure_libmpv()
        .await
        .map_err(|e| OpError::new(&e.to_string()))?;

    let player = VideoPlayer::new().map_err(|e| OpError::new(&e.to_string()))?;
    player
        .load(path.to_str().unwrap())
        .map_err(|e| OpError::new(&e.to_string()))?;

    if let Some(vol) = opts.volume {
        player
            .set_volume(vol as f64)
            .map_err(|e| OpError::new(&e.to_string()))?;
    }

    let player = Arc::new(player);
    let content = VideoContent::new(player.clone());

    // Default size if not specified? Video player should probably know its size.
    // But for now we can default to something or use window options.
    let size = opts
        .window
        .size
        .map(|s| (s.width, s.height))
        .unwrap_or((800, 600));

    let window_opts = crate::gui::window_manager::WindowOptions {
        size: Some(size),
        opacity: opts.window.opacity.unwrap_or(1.0),
        always_on_top: opts.window.always_on_top.unwrap_or(false),
        click_through: opts.window.click_through.unwrap_or(false),
        position: opts.window.position.map(|p| (p.x, p.y)),
        decorations: false,
        timeout: opts.duration.map(std::time::Duration::from_secs),
        ..Default::default()
    };

    let handle = gui_controller
        .create_window(window_opts)
        .map_err(|e| OpError::new(&e.to_string()))?;

    gui_controller
        .set_content(handle, Box::new(content))
        .map_err(|e| OpError::new(&e.to_string()))?;

    if opts.autoplay.unwrap_or(true) {
        player.play().map_err(|e| OpError::new(&e.to_string()))?;
    }

    Ok(handle.0.to_string())
}

pub const TS_SOURCE: &str = include_str!("js/video.ts");

deno_core::extension!(goon_video, ops = [op_show_video],);
