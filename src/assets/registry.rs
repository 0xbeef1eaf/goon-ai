use crate::assets::types::Asset;

#[derive(Debug, Default)]
pub struct AssetRegistry {
    pub images: Vec<Asset>,
    pub videos: Vec<Asset>,
    pub audio: Vec<Asset>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, asset: Asset) {
        match asset {
            Asset::Image(_) => self.images.push(asset),
            Asset::Video(_) => self.videos.push(asset),
            Asset::Audio(_) => self.audio.push(asset),
        }
    }
}
