use crate::assets::registry::AssetRegistry;
use crate::assets::types::{Asset, AudioAsset, ImageAsset, VideoAsset};
use crate::config::pack::PackConfig;
use anyhow::Result;
use std::path::Path;

#[allow(dead_code)]
pub struct AssetLoader;

impl AssetLoader {
    #[allow(dead_code)]
    pub fn load(pack_config: &PackConfig, pack_name: &str) -> Result<AssetRegistry> {
        let mut registry = AssetRegistry::new();
        let base_path = Path::new("packs").join(pack_name);

        if let Some(images) = &pack_config.assets.image {
            for img in images {
                let path = base_path.join(&img.path);
                registry.add(Asset::Image(ImageAsset {
                    path,
                    tags: img.tags.clone(),
                }));
            }
        }

        if let Some(videos) = &pack_config.assets.video {
            for vid in videos {
                let path = base_path.join(&vid.path);
                registry.add(Asset::Video(VideoAsset {
                    path,
                    tags: vid.tags.clone(),
                    duration: None, // Duration loading can be added later
                }));
            }
        }

        if let Some(audio) = &pack_config.assets.audio {
            for aud in audio {
                let path = base_path.join(&aud.path);
                registry.add(Asset::Audio(AudioAsset {
                    path,
                    tags: aud.tags.clone(),
                    duration: None,
                }));
            }
        }

        Ok(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::pack::{Asset as ConfigAsset, Assets, PackMeta};

    #[test]
    fn test_load_assets() {
        let pack_config = PackConfig {
            meta: PackMeta {
                name: "Test Pack".to_string(),
                version: "1.0.0".to_string(),
                permissions: vec![],
            },
            moods: vec![],
            assets: Assets {
                image: Some(vec![ConfigAsset {
                    path: "img/1.jpg".to_string(),
                    tags: vec!["tag1".to_string()],
                }]),
                video: Some(vec![ConfigAsset {
                    path: "vid/1.mp4".to_string(),
                    tags: vec!["tag2".to_string()],
                }]),
                audio: None,
            },
        };

        let registry = AssetLoader::load(&pack_config, "Test Pack").unwrap();

        assert_eq!(registry.images.len(), 1);
        assert_eq!(registry.videos.len(), 1);
        assert_eq!(registry.audio.len(), 0);

        if let Asset::Image(img) = &registry.images[0] {
            assert!(img.path.ends_with("packs/Test Pack/img/1.jpg"));
            assert_eq!(img.tags, vec!["tag1"]);
        } else {
            panic!("Expected ImageAsset");
        }
    }
}
