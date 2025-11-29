use anyhow::Result;
use image::{AnimationDecoder, RgbaImage};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

#[derive(Clone)]
pub struct Frame {
    pub buffer: RgbaImage,
    pub delay: Duration,
}

#[derive(Clone)]
pub struct Animation {
    pub frames: Vec<Frame>,
    pub total_duration: Duration,
}

impl Animation {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let decoder = image::codecs::gif::GifDecoder::new(reader)?;
        let frames = decoder.into_frames();
        let frames = frames.collect_frames()?;

        let mut anim_frames = Vec::new();
        let mut total_duration = Duration::ZERO;

        for frame in frames {
            let delay = Duration::from(frame.delay());
            let buffer = frame.into_buffer();

            anim_frames.push(Frame { buffer, delay });
            total_duration += delay;
        }

        Ok(Self {
            frames: anim_frames,
            total_duration,
        })
    }
}
