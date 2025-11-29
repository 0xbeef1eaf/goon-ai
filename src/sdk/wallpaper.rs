use crate::assets::registry::AssetRegistry;
use crate::assets::selector::AssetSelector;
use crate::assets::types::Asset;
use crate::config::pack::Mood;
use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
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
pub struct WallpaperOptions {
    tags: Option<Vec<String>>,
}

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

    let path = match asset {
        Asset::Wallpaper(w) => &w.path,
        _ => return Err(OpError::new("Selected asset is not a wallpaper")),
    };

    println!("Setting wallpaper: {:?} with options: {:?}", path, opts);
    Ok(())
}

pub const TS_SOURCE: &str = include_str!("js/wallpaper.ts");

deno_core::extension!(goon_wallpaper, ops = [op_set_wallpaper],);
