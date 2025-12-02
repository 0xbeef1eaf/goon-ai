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
use tracing::{info, warn};
use ts_rs::TS;

#[derive(Deserialize, Debug, Default, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
/// Options for displaying an image
pub struct ImageOptions {
    /// Tags to filter the images by
    pub tags: Option<Vec<String>>,
    /// Duration to display the image in seconds
    pub duration: Option<u64>,
    #[serde(flatten)]
    pub window: WindowOptions,
}

#[op2(async)]
#[string]
pub async fn op_show_image(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<ImageOptions>,
) -> Result<String, OpError> {
    let (_window_spawner, registry, mood) = {
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
        Asset::Image(img) => &img.path,
        _ => return Err(OpError::new("Selected asset is not an image")),
    };

    // TODO: Implement image window spawning in the new architecture
    // For now, log the image that would be shown
    info!("Would show image: {:?}", path);
    warn!("Image window spawning not yet implemented in new architecture");

    // Return a placeholder handle
    Ok(uuid::Uuid::new_v4().to_string())
}

pub const TS_SOURCE: &str = include_str!("js/image.ts");

deno_core::extension!(goon_image, ops = [op_show_image],);
