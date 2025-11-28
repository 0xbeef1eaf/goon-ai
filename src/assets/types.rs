use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Asset {
    Image(ImageAsset),
    Video(VideoAsset),
    Audio(AudioAsset),
    Hypno(HypnoAsset),
    Wallpaper(WallpaperAsset),
}

impl Asset {
    #[allow(dead_code)]
    pub fn get_path(&self) -> &PathBuf {
        match self {
            Asset::Image(a) => &a.path,
            Asset::Video(a) => &a.path,
            Asset::Audio(a) => &a.path,
            Asset::Hypno(a) => &a.path,
            Asset::Wallpaper(a) => &a.path,
        }
    }

    #[allow(dead_code)]
    pub fn get_tags(&self) -> &Vec<String> {
        match self {
            Asset::Image(a) => &a.tags,
            Asset::Video(a) => &a.tags,
            Asset::Audio(a) => &a.tags,
            Asset::Hypno(a) => &a.tags,
            Asset::Wallpaper(a) => &a.tags,
        }
    }

    #[allow(dead_code)]
    pub fn load_data(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct ImageAsset {
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct VideoAsset {
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub duration: Option<Duration>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct AudioAsset {
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct HypnoAsset {
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub is_animated: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct WallpaperAsset {
    pub path: PathBuf,
    pub tags: Vec<String>,
}
