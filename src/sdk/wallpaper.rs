use crate::assets::registry::AssetRegistry;
use crate::assets::selector::AssetSelector;
use crate::assets::types::Asset;
use crate::config::pack::Mood;
use crate::media::wallpaper::{PlatformWallpaperSetter, WallpaperSetter};
use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
use deno_core::OpState;
use deno_core::op2;
use serde::Deserialize;
use serde_json;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use std::sync::Arc;
use ts_rs::TS;

#[derive(Deserialize, Debug, Default, TS)]
#[serde(rename_all = "camelCase")]
/// Options for setting the desktop wallpaper
pub struct WallpaperOptions {
    /// A list of additional tags to filter wallpaper images by, they will be filtered by mood tags already
    tags: Option<Vec<String>>,
}

/// Sets the desktop wallpaper to an image from the pack.
///
/// @param options - Optional configuration including tags for asset selection.
#[op2(async)]
pub async fn op_set_wallpaper(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<(), OpError> {
    let (registry, mood) = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "wallpaper")?;
        let registry = state.borrow::<Arc<AssetRegistry>>().clone();
        let mood = state.borrow::<Mood>().clone();
        (registry, mood)
    };

    let opts: WallpaperOptions = if let Some(o) = options {
        serde_json::from_value(o).map_err(|e| OpError::new(&e.to_string()))?
    } else {
        WallpaperOptions::default()
    };

    let tags = opts.tags.clone().unwrap_or_default();
    let selector = AssetSelector::new(&registry);

    let asset = selector
        .select_wallpaper(&mood, &tags)
        .ok_or_else(|| OpError::new("No wallpaper found matching tags"))?;

    let path_to_set = match asset {
        Asset::Wallpaper(w) => w.path.clone(),
        _ => return Err(OpError::new("Selected asset is not a wallpaper")),
    };

    // Create persistent directory
    let data_dir =
        dirs::data_local_dir().ok_or_else(|| OpError::new("Could not find data directory"))?;
    let wallpaper_dir = data_dir.join("goon-ai").join("wallpapers");
    fs::create_dir_all(&wallpaper_dir)
        .map_err(|e| OpError::new(&format!("Failed to create wallpaper directory: {}", e)))?;

    // Copy file
    let file_name = path_to_set
        .file_name()
        .ok_or_else(|| OpError::new("Invalid wallpaper path"))?;
    let target_path = wallpaper_dir.join(file_name);
    fs::copy(&path_to_set, &target_path)
        .map_err(|e| OpError::new(&format!("Failed to copy wallpaper: {}", e)))?;

    let setter = PlatformWallpaperSetter;
    setter
        .set_wallpaper(&target_path)
        .map_err(|e| OpError::new(&format!("Failed to set wallpaper: {}", e)))?;

    Ok(())
}

deno_core::extension!(goon_wallpaper, ops = [op_set_wallpaper],);
