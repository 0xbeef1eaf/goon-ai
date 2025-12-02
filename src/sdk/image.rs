use crate::assets::registry::AssetRegistry;
use crate::assets::selector::AssetSelector;
use crate::assets::types::Asset;
use crate::config::pack::Mood;
use crate::gui::WindowSpawnerHandle;
use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
use crate::sdk::types::WindowOptions;
use deno_core::OpState;
use deno_core::op2;
use serde::Deserialize;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tracing::{error, info};
use ts_rs::TS;

#[derive(Deserialize, Debug, Default, TS)]
#[serde(rename_all = "camelCase")]
/// Options for displaying an image
pub struct ImageOptions {
    /// A list of additional tags to filter images by, they will be filtered by mood tags already
    pub tags: Option<Vec<String>>,
    /// Duration to display the image in seconds, after this the window will be closed automatically
    pub duration: Option<u64>,
    /// Window configuration options
    pub window: Option<WindowOptions>,
}

/// Displays an image in a new window.
///
/// Returns a handle ID that can be used to control the window (move, resize, close).
///
/// @param options - Optional configuration including tags for asset selection,
///                  window position, size, and opacity.
/// @returns A unique handle ID string for controlling this image window.
#[op2(async)]
#[string]
pub async fn op_show_image(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<ImageOptions>,
) -> Result<String, OpError> {
    let (window_spawner, registry, mood) = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "image")?;
        let spawner = state.borrow::<WindowSpawnerHandle>().clone();
        let registry = state.borrow::<Arc<AssetRegistry>>().clone();
        let mood = state.borrow::<Mood>().clone();
        (spawner, registry, mood)
    };

    let opts = options.unwrap_or_default();

    let tags = opts.tags.unwrap_or_default();
    let selector = AssetSelector::new(&registry);

    let asset = selector
        .select_image(&mood, &tags)
        .ok_or_else(|| OpError::new("No image found matching tags"))?;

    let path = match asset {
        Asset::Image(img) => img.path.clone(),
        _ => return Err(OpError::new("Selected asset is not an image")),
    };

    info!("Spawning image window: {:?}", path);

    // Get window dimensions from options
    let window = opts.window.as_ref();
    let width = window.and_then(|w| w.size.as_ref()).map(|s| s.width);
    let height = window.and_then(|w| w.size.as_ref()).map(|s| s.height);
    let opacity = window.and_then(|w| w.opacity).unwrap_or(1.0);

    // Spawn the image window
    let handle = window_spawner
        .spawn_image(path, width, height, opacity)
        .map_err(|e| {
            error!("Failed to spawn image window: {}", e);
            OpError::new(&e.to_string())
        })?;

    Ok(handle.0.to_string())
}

deno_core::extension!(goon_image, ops = [op_show_image],);
