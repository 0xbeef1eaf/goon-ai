use anyhow::{Context, Result};
use libmpv2::Mpv;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub struct VideoPlayer {
    mpv: Mpv,
    playing: Arc<AtomicBool>,
}

impl VideoPlayer {
    pub fn new() -> Result<Self> {
        let mpv = Mpv::new()
            .map_err(|e| anyhow::anyhow!(e.to_string()))
            .context("Failed to initialize MPV")?;

        // Set default properties
        mpv.set_property("vo", "libmpv")
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        mpv.set_property("hwdec", "auto")
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        Ok(Self {
            mpv,
            playing: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn load(&self, path: &str) -> Result<()> {
        self.mpv
            .command("loadfile", &[path, "replace"])
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        self.playing.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn play(&self) -> Result<()> {
        self.mpv
            .set_property("pause", false)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        self.playing.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.mpv
            .set_property("pause", true)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        self.playing.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.mpv
            .command("stop", &[])
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        self.playing.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub fn set_volume(&self, volume: f64) -> Result<()> {
        self.mpv
            .set_property("volume", volume)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(())
    }

    pub fn get_mpv(&self) -> &Mpv {
        &self.mpv
    }
}
