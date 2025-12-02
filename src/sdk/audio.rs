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
use uuid::Uuid;

/// Parse a string handle ID into an AudioHandle
fn parse_audio_handle(handle_id: &str) -> Result<AudioHandle, OpError> {
    let uuid = Uuid::parse_str(handle_id).map_err(|_| OpError::new("Invalid audio handle ID"))?;
    Ok(AudioHandle(uuid))
}

#[derive(Deserialize, Debug, Default, TS)]
#[serde(rename_all = "camelCase")]
/// Options for playing audio
pub struct AudioOptions {
    /// A list of additional tags to filter audio files by, they will be filtered by mood tags already
    tags: Option<Vec<String>>,
    /// Whether to loop the audio continuously
    loop_: Option<bool>,
    /// Volume level from 0.0 (muted) to 1.0 (full volume)
    volume: Option<f32>,
    /// Duration to play the audio in seconds, after this playback will stop automatically
    duration: Option<f64>,
}

#[op2(async)]
#[string]
pub async fn op_play_audio(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<String, OpError> {
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

    Ok(handle.0.to_string())
}

/// Stops audio playback for the given handle.
///
/// Once stopped, the audio cannot be resumed. Use pause() if you want to resume later.
///
/// @param handle - The handle ID returned from play().
#[op2(async)]
pub async fn op_stop_audio(
    state: Rc<RefCell<OpState>>,
    #[string] handle_id: String,
) -> Result<(), OpError> {
    let handle = parse_audio_handle(&handle_id)?;
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

/// Pauses audio playback for the given handle.
///
/// The audio can be resumed later with resume().
///
/// @param handle - The handle ID returned from play().
#[op2(async)]
pub async fn op_pause_audio(
    state: Rc<RefCell<OpState>>,
    #[string] handle_id: String,
) -> Result<(), OpError> {
    let handle = parse_audio_handle(&handle_id)?;
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

/// Resumes audio playback for a paused handle.
///
/// @param handle - The handle ID returned from play().
#[op2(async)]
pub async fn op_resume_audio(
    state: Rc<RefCell<OpState>>,
    #[string] handle_id: String,
) -> Result<(), OpError> {
    let handle = parse_audio_handle(&handle_id)?;
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

/// Sets the volume for a playing audio handle.
///
/// @param handle - The handle ID returned from play().
/// @param volume - Volume level from 0.0 (silent) to 1.0 (full volume).
#[op2(async)]
pub async fn op_set_audio_volume(
    state: Rc<RefCell<OpState>>,
    #[string] handle_id: String,
    volume: f32,
) -> Result<(), OpError> {
    let handle = parse_audio_handle(&handle_id)?;
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
