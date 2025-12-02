use crate::assets::registry::AssetRegistry;
use crate::assets::selector::AssetSelector;
use crate::assets::types::Asset;
use crate::config::pack::Mood;
use crate::media::audio::manager::{AudioHandle, AudioManager};
use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
use deno_core::OpState;
use deno_core::op2;
use serde::Deserialize;
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use ts_rs::TS;

#[derive(Deserialize, Debug, Default, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AudioOptions {
    tags: Option<Vec<String>>,
    loop_: Option<bool>,
    volume: Option<f32>,
    duration: Option<f64>,
}

#[op2(async)]
#[serde]
pub async fn op_play_audio(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<AudioHandle, OpError> {
    let (registry, mood, audio_manager) = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "audio")?;
        let registry = state.borrow::<Arc<AssetRegistry>>().clone();
        let mood = state.borrow::<Mood>().clone();
        let audio_manager = state.try_borrow::<Arc<Mutex<AudioManager>>>().cloned();
        (registry, mood, audio_manager)
    };

    let audio_manager =
        audio_manager.ok_or_else(|| OpError::new("Audio system not initialized"))?;

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

    let volume = opts.volume.unwrap_or(1.0);
    let duration = opts.duration.map(std::time::Duration::from_secs_f64);

    let handle = {
        let mut manager = audio_manager
            .lock()
            .map_err(|_| OpError::new("Failed to lock audio manager"))?;
        manager
            .play_audio(path.clone(), volume, duration)
            .map_err(|e| OpError::new(&e.to_string()))?
    };

    Ok(handle)
}

#[op2(async)]
pub async fn op_stop_audio(
    state: Rc<RefCell<OpState>>,
    #[serde] handle: AudioHandle,
) -> Result<(), OpError> {
    let audio_manager = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "audio")?;
        state.try_borrow::<Arc<Mutex<AudioManager>>>().cloned()
    };

    if let Some(manager) = audio_manager {
        let mut manager = manager
            .lock()
            .map_err(|_| OpError::new("Failed to lock audio manager"))?;
        manager.stop_audio(handle);
    }
    Ok(())
}

#[op2(async)]
pub async fn op_pause_audio(
    state: Rc<RefCell<OpState>>,
    #[serde] handle: AudioHandle,
) -> Result<(), OpError> {
    let audio_manager = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "audio")?;
        state.try_borrow::<Arc<Mutex<AudioManager>>>().cloned()
    };

    if let Some(manager) = audio_manager {
        let manager = manager
            .lock()
            .map_err(|_| OpError::new("Failed to lock audio manager"))?;
        manager.pause_audio(handle);
    }
    Ok(())
}

#[op2(async)]
pub async fn op_resume_audio(
    state: Rc<RefCell<OpState>>,
    #[serde] handle: AudioHandle,
) -> Result<(), OpError> {
    let audio_manager = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "audio")?;
        state.try_borrow::<Arc<Mutex<AudioManager>>>().cloned()
    };

    if let Some(manager) = audio_manager {
        let manager = manager
            .lock()
            .map_err(|_| OpError::new("Failed to lock audio manager"))?;
        manager.resume_audio(handle);
    }
    Ok(())
}

#[op2(async)]
pub async fn op_set_audio_volume(
    state: Rc<RefCell<OpState>>,
    #[serde] handle: AudioHandle,
    volume: f32,
) -> Result<(), OpError> {
    let audio_manager = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "audio")?;
        state.try_borrow::<Arc<Mutex<AudioManager>>>().cloned()
    };

    if let Some(manager) = audio_manager {
        let manager = manager
            .lock()
            .map_err(|_| OpError::new("Failed to lock audio manager"))?;
        manager.set_volume(handle, volume);
    }
    Ok(())
}

deno_core::extension!(
    goon_audio,
    ops = [
        op_play_audio,
        op_stop_audio,
        op_pause_audio,
        op_resume_audio,
        op_set_audio_volume
    ],
);
