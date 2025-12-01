use crate::assets::registry::AssetRegistry;
use crate::assets::types::{
    Asset, AudioAsset, HypnoAsset, ImageAsset, VideoAsset, WallpaperAsset, WebsiteAsset,
};
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
                    width: 0,
                    height: 0,
                }));
            }
        }

        if let Some(videos) = &pack_config.assets.video {
            for vid in videos {
                let path = base_path.join(&vid.path);
                registry.add(Asset::Video(VideoAsset {
                    path,
                    tags: vid.tags.clone(),
                    duration: None,
                    width: 0,
                    height: 0,
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

        if let Some(hypnos) = &pack_config.assets.hypno {
            for hyp in hypnos {
                let path = base_path.join(&hyp.path);
                registry.add(Asset::Hypno(HypnoAsset {
                    path,
                    tags: hyp.tags.clone(),
                    is_animated: true,
                }));
            }
        }

        if let Some(wallpapers) = &pack_config.assets.wallpaper {
            for wall in wallpapers {
                let path = base_path.join(&wall.path);
                registry.add(Asset::Wallpaper(WallpaperAsset {
                    path,
                    tags: wall.tags.clone(),
                }));
            }
        }

        if let Some(websites) = &pack_config.websites {
            for site in websites {
                registry.add(Asset::Website(WebsiteAsset {
                    name: site.name.clone(),
                    url: site.url.clone(),
                    description: site.description.clone(),
                    tags: site.tags.clone(),
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
                hypno: None,
                wallpaper: None,
            },
            websites: None,
            prompts: None,
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
