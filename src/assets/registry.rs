use crate::assets::types::Asset;

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct AssetRegistry {
    pub images: Vec<Asset>,
    pub videos: Vec<Asset>,
    pub audio: Vec<Asset>,
    pub hypnos: Vec<Asset>,
    pub wallpapers: Vec<Asset>,
}

impl AssetRegistry {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn add(&mut self, asset: Asset) {
        match asset {
            Asset::Image(_) => self.images.push(asset),
            Asset::Video(_) => self.videos.push(asset),
            Asset::Audio(_) => self.audio.push(asset),
            Asset::Hypno(_) => self.hypnos.push(asset),
            Asset::Wallpaper(_) => self.wallpapers.push(asset),
        }
    }
}
