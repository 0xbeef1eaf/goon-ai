use crate::assets::registry::AssetRegistry;
use crate::assets::selector::AssetSelector;
use crate::assets::types::Asset;
use crate::config::pack::Mood;
use crate::media::video::manager::VideoManager;
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
use tokio::sync::Mutex;
use ts_rs::TS;

#[derive(Deserialize, Debug, Default, TS)]
#[serde(rename_all = "camelCase")]
/// Options for playing a video
pub struct VideoOptions {
    /// A list of additional tags to filter videos by, they will be filtered by mood tags already
    pub tags: Option<Vec<String>>,
    /// Whether to loop the video continuously
    pub loop_: Option<bool>,
    /// Volume level from 0.0 (muted) to 1.0 (full volume)
    pub volume: Option<f32>,
    /// Whether to start playing automatically
    pub autoplay: Option<bool>,
    /// Duration to play the video in seconds, after this the window will be closed automatically
    pub duration: Option<u64>,
    /// Window configuration options
    pub window: Option<WindowOptions>,
}

/// Plays a video in a new window.
///
/// Returns a handle ID that can be used to control the window (move, resize, close).
///
/// @param options - Optional configuration including tags for asset selection,
///                  window position, size, looping, and muting options.
/// @returns A unique handle ID string for controlling this video window.
#[op2(async)]
#[string]
pub async fn op_show_video(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<String, OpError> {
    let (registry, mood, video_manager) = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "video")?;
        let registry = state.borrow::<Arc<AssetRegistry>>().clone();
        let mood = state.borrow::<Mood>().clone();
        let video_manager = state.try_borrow::<Arc<Mutex<VideoManager>>>().cloned();
        (registry, mood, video_manager)
    };

    let video_manager =
        video_manager.ok_or_else(|| OpError::new("Video system not initialized"))?;

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

    let handle = {
        let mut manager = video_manager.lock().await;
        manager
            .spawn_video(path.clone(), &opts)
            .await
            .map_err(|e| OpError::new(&e.to_string()))?
    };

    Ok(handle.0.to_string())
}

deno_core::extension!(goon_video, ops = [op_show_video],);
