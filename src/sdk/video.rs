use crate::assets::registry::AssetRegistry;
use crate::assets::selector::AssetSelector;
use crate::assets::types::Asset;
use crate::config::pack::Mood;
use crate::gui::WindowSpawnerHandle;
use crate::permissions::Permission;
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
use uuid::Uuid;

/// Parse a string handle ID into a window handle UUID
fn parse_video_handle(handle_id: &str) -> Result<Uuid, OpError> {
    Uuid::parse_str(handle_id).map_err(|_| OpError::new("Invalid video handle ID"))
}

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
/// Returns a handle object that can be used to control the window.
/// The returned handle has a `.close()` method to close the window.
///
/// @param options - Optional configuration including tags for asset selection,
///                  window position, size, looping, and muting options.
/// @returns A unique handle object for controlling this video window.
#[op2(async)]
#[string]
pub async fn op_show_video(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<String, OpError> {
    let (registry, mood, window_spawner) = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, Permission::Video)?;
        let registry = state.borrow::<Arc<AssetRegistry>>().clone();
        let mood = state.borrow::<Mood>().clone();
        let window_spawner = state.borrow::<WindowSpawnerHandle>().clone();
        (registry, mood, window_spawner)
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
        Asset::Video(vid) => vid.path.clone(),
        _ => return Err(OpError::new("Selected asset is not a video")),
    };

    tracing::info!("Showing video: {:?} with options: {:?}", path, opts);

    let window = opts.window.as_ref();
    let width = window.and_then(|w| w.size.as_ref()).map(|s| s.width);
    let height = window.and_then(|w| w.size.as_ref()).map(|s| s.height);
    let opacity = window.and_then(|w| w.opacity).unwrap_or(1.0);
    let loop_playback = opts.loop_.unwrap_or(false);
    let volume = opts.volume.unwrap_or(1.0);

    let handle = window_spawner
        .spawn_video(path, width, height, opacity, loop_playback, volume)
        .map_err(|e| OpError::new(&e.to_string()))?;

    Ok(handle.0.to_string())
}

/// Pauses video playback for the given handle.
///
/// @param handle - The handle ID returned from play().
#[op2(async)]
pub async fn op_pause_video(
    state: Rc<RefCell<OpState>>,
    #[string] handle_id: String,
) -> Result<(), OpError> {
    let handle = parse_video_handle(&handle_id)?;
    let window_spawner = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, Permission::Video)?;
        state.borrow::<WindowSpawnerHandle>().clone()
    };

    window_spawner
        .pause_video(crate::gui::windows::WindowHandle(handle))
        .map_err(|e| OpError::new(&e.to_string()))?;

    Ok(())
}

/// Resumes video playback for a paused handle.
///
/// @param handle - The handle ID returned from play().
#[op2(async)]
pub async fn op_resume_video(
    state: Rc<RefCell<OpState>>,
    #[string] handle_id: String,
) -> Result<(), OpError> {
    let handle = parse_video_handle(&handle_id)?;
    let window_spawner = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, Permission::Video)?;
        state.borrow::<WindowSpawnerHandle>().clone()
    };

    window_spawner
        .resume_video(crate::gui::windows::WindowHandle(handle))
        .map_err(|e| OpError::new(&e.to_string()))?;

    Ok(())
}

deno_core::extension!(
    goon_video,
    ops = [op_show_video, op_pause_video, op_resume_video],
);
