use anyhow::Result;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;

pub struct AudioPlayer {
    sink: Sink,
    file_path: PathBuf,
}

impl AudioPlayer {
    pub fn new(stream_handle: &OutputStreamHandle, file_path: PathBuf) -> Result<Self> {
        let sink = Sink::try_new(stream_handle)?;
        Ok(Self { sink, file_path })
    }

    pub fn play(&self, duration: Option<Duration>) -> Result<()> {
        let file = File::open(&self.file_path)?;
        let source = Decoder::new(BufReader::new(file))?;
        if let Some(d) = duration {
            self.sink.append(source.take_duration(d));
        } else {
            self.sink.append(source);
        }
        Ok(())
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn resume(&self) {
        self.sink.play();
    }

    pub fn stop(&self) {
        self.sink.stop();
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn is_finished(&self) -> bool {
        self.sink.empty()
    }
}
