use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Asset {
    Image(ImageAsset),
    Video(VideoAsset),
    Audio(AudioAsset),
}

impl Asset {
    pub fn get_path(&self) -> &PathBuf {
        match self {
            Asset::Image(a) => &a.path,
            Asset::Video(a) => &a.path,
            Asset::Audio(a) => &a.path,
        }
    }

    pub fn get_tags(&self) -> &Vec<String> {
        match self {
            Asset::Image(a) => &a.tags,
            Asset::Video(a) => &a.tags,
            Asset::Audio(a) => &a.tags,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageAsset {
    pub path: PathBuf,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoAsset {
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioAsset {
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub duration: Option<Duration>,
}
