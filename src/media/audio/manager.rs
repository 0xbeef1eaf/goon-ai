use super::player::AudioPlayer;
use anyhow::Result;
use rodio::OutputStreamHandle;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioHandle(pub Uuid);

pub struct AudioManager {
    stream_handle: OutputStreamHandle,
    players: HashMap<AudioHandle, AudioPlayer>,
    play_order: Vec<AudioHandle>,
    max_concurrent: usize,
}

impl AudioManager {
    pub fn new(stream_handle: OutputStreamHandle, max_concurrent: usize) -> Self {
        Self {
            stream_handle,
            players: HashMap::new(),
            play_order: Vec::new(),
            max_concurrent,
        }
    }

    pub fn play_audio(
        &mut self,
        file_path: PathBuf,
        volume: f32,
        duration: Option<Duration>,
    ) -> Result<AudioHandle> {
        // Clean up finished players first
        self.cleanup_finished();

        // Enforce limit
        if self.players.len() >= self.max_concurrent {
            let oldest = self.play_order.first().copied();
            if let Some(oldest) = oldest {
                self.stop_audio(oldest);
            }
        }

        let player = AudioPlayer::new(&self.stream_handle, file_path)?;
        player.set_volume(volume);
        player.play(duration)?;

        let handle = AudioHandle(Uuid::new_v4());
        self.players.insert(handle, player);
        self.play_order.push(handle);

        Ok(handle)
    }

    pub fn stop_audio(&mut self, handle: AudioHandle) {
        if let Some(player) = self.players.remove(&handle) {
            player.stop();
        }
        self.play_order.retain(|&h| h != handle);
    }

    pub fn pause_audio(&self, handle: AudioHandle) {
        if let Some(player) = self.players.get(&handle) {
            player.pause();
        }
    }

    pub fn resume_audio(&self, handle: AudioHandle) {
        if let Some(player) = self.players.get(&handle) {
            player.resume();
        }
    }

    pub fn set_volume(&self, handle: AudioHandle, volume: f32) {
        if let Some(player) = self.players.get(&handle) {
            player.set_volume(volume);
        }
    }

    fn cleanup_finished(&mut self) {
        let finished: Vec<AudioHandle> = self
            .players
            .iter()
            .filter(|(_, p)| p.is_finished())
            .map(|(&h, _)| h)
            .collect();

        for handle in finished {
            self.stop_audio(handle);
        }
    }
}
