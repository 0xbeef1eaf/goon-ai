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
pub struct WebsiteOptions {
    #[ts(optional)]
    tags: Option<Vec<String>>,
}

#[op2(async)]
pub async fn op_open_website(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<(), OpError> {
    let (registry, mood) = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "website")?;
        let registry = state.borrow::<Arc<AssetRegistry>>().clone();
        let mood = state.borrow::<Mood>().clone();
        (registry, mood)
    };

    let opts: WebsiteOptions = if let Some(o) = options {
        serde_json::from_value(o).map_err(|e| OpError::new(&e.to_string()))?
    } else {
        WebsiteOptions::default()
    };

    let tags = opts.tags.clone().unwrap_or_default();
    let selector = AssetSelector::new(&registry);

    let asset = selector
        .select_website(&mood, &tags)
        .ok_or_else(|| OpError::new("No website found matching tags"))?;

    let url = match asset {
        Asset::Website(w) => &w.url,
        _ => return Err(OpError::new("Selected asset is not a website")),
    };

    open::that(url).map_err(|e| OpError::new(&format!("Failed to open website: {}", e)))?;

    Ok(())
}

pub fn get_source() -> String {
    let decl = WebsiteOptions::decl();
    format!("{}\n{}", decl, include_str!("js/website.ts"))
}

deno_core::extension!(goon_website, ops = [op_open_website],);
