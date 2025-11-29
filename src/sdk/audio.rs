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
pub struct AudioOptions {
    tags: Option<Vec<String>>,
    loop_: Option<bool>,
    volume: Option<f32>,
}

#[op2(async)]
pub async fn op_play_audio(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<(), OpError> {
    let (registry, mood) = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "audio")?;
        let registry = state.borrow::<Arc<AssetRegistry>>().clone();
        let mood = state.borrow::<Mood>().clone();
        (registry, mood)
    };

    let opts: AudioOptions = if let Some(o) = options {
        serde_json::from_value(o).map_err(|e| OpError::new(&e.to_string()))?
    } else {
        AudioOptions::default()
    };

    let tags = opts.tags.clone().unwrap_or_default();
    let selector = AssetSelector::new(&registry);

    let asset = selector
        .select_audio(&mood, &tags)
        .ok_or_else(|| OpError::new("No audio found matching tags"))?;

    let path = match asset {
        Asset::Audio(aud) => &aud.path,
        _ => return Err(OpError::new("Selected asset is not an audio file")),
    };

    println!("Playing audio: {:?} with options: {:?}", path, opts);
    Ok(())
}

pub const TS_SOURCE: &str = include_str!("js/audio.ts");

deno_core::extension!(goon_audio, ops = [op_play_audio],);
