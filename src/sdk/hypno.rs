use crate::assets::registry::AssetRegistry;
use crate::assets::selector::AssetSelector;
use crate::assets::types::Asset;
use crate::config::pack::Mood;
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
pub struct HypnoOptions {
    pub tags: Option<Vec<String>>,
    pub duration: Option<u64>,
    #[serde(flatten)]
    pub window: WindowOptions,
}

#[op2(async)]
pub async fn op_show_hypno(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<u32, OpError> {
    let (registry, mood) = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "hypno")?;
        let registry = state.borrow::<Arc<AssetRegistry>>().clone();
        let mood = state.borrow::<Mood>().clone();
        (registry, mood)
    };

    let opts: HypnoOptions = if let Some(o) = options {
        serde_json::from_value(o).map_err(|e| OpError::new(&e.to_string()))?
    } else {
        HypnoOptions::default()
    };

    let tags = opts.tags.clone().unwrap_or_default();
    let selector = AssetSelector::new(&registry);

    let asset = selector
        .select_hypno(&mood, &tags)
        .ok_or_else(|| OpError::new("No hypno pattern found matching tags"))?;

    let path = match asset {
        Asset::Hypno(h) => &h.path,
        _ => return Err(OpError::new("Selected asset is not a hypno pattern")),
    };

    println!("Showing hypno: {:?} with options: {:?}", path, opts);
    Ok(3)
}

deno_core::extension!(goon_hypno, ops = [op_show_hypno],);
