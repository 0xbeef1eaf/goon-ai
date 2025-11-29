use super::player::VideoPlayer;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoHandle(pub Uuid);

pub struct VideoManager {
    players: HashMap<VideoHandle, Arc<Mutex<VideoPlayer>>>,
    max_concurrent: usize,
}

impl VideoManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            players: HashMap::new(),
            max_concurrent,
        }
    }

    pub async fn spawn_video(
        &mut self,
        file_path: PathBuf,
        options: &crate::sdk::video::VideoOptions,
    ) -> Result<VideoHandle> {
        // Enforce limit
        if self.players.len() >= self.max_concurrent {
            // Simple strategy: reject or remove oldest?
            // Let's remove oldest for now if we tracked order, but HashMap is unordered.
            // For simplicity, just reject or pick one.
            // Better: track order. But for now, let's just stop one.
            if let Some(key) = self.players.keys().next().copied() {
                self.stop_video(key).await;
            }
        }

        let player = VideoPlayer::spawn(file_path, options).await?;
        let handle = VideoHandle(Uuid::new_v4());
        self.players.insert(handle, Arc::new(Mutex::new(player)));
        Ok(handle)
    }

    pub async fn stop_video(&mut self, handle: VideoHandle) {
        if let Some(player) = self.players.remove(&handle) {
            let mut p = player.lock().await;
            let _ = p.stop().await;
        }
    }
}
