use anyhow::Result;
use image::ImageReader;
use std::path::Path;

pub fn load_image<P: AsRef<Path>>(path: P) -> Result<image::RgbaImage> {
    let img = ImageReader::open(path)?.decode()?;
    Ok(img.to_rgba8())
}
