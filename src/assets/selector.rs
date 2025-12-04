use crate::assets::registry::AssetRegistry;
use crate::assets::types::Asset;
use crate::config::pack::Mood;
use rand::prelude::IndexedRandom;

#[allow(dead_code)]
pub struct AssetSelector<'a> {
    registry: &'a AssetRegistry,
}

impl<'a> AssetSelector<'a> {
    #[allow(dead_code)]
    pub fn new(registry: &'a AssetRegistry) -> Self {
        Self { registry }
    }

    #[allow(dead_code)]
    pub fn select_image(&self, mood: &Mood, tags: &[String]) -> Option<&Asset> {
        self.select_from(&self.registry.images, mood, tags)
    }

    #[allow(dead_code)]
    pub fn select_video(&self, mood: &Mood, tags: &[String]) -> Option<&Asset> {
        self.select_from(&self.registry.videos, mood, tags)
    }

    #[allow(dead_code)]
    pub fn select_audio(&self, mood: &Mood, tags: &[String]) -> Option<&Asset> {
        self.select_from(&self.registry.audio, mood, tags)
    }

    #[allow(dead_code)]
    pub fn select_hypno(&self, mood: &Mood, tags: &[String]) -> Option<&Asset> {
        self.select_from(&self.registry.hypnos, mood, tags)
    }

    #[allow(dead_code)]
    pub fn select_wallpaper(&self, mood: &Mood, tags: &[String]) -> Option<&Asset> {
        self.select_from(&self.registry.wallpapers, mood, tags)
    }

    #[allow(dead_code)]
    pub fn select_website(&self, mood: &Mood, tags: &[String]) -> Option<&Asset> {
        self.select_from(&self.registry.websites, mood, tags)
    }

    fn select_from(&self, assets: &'a [Asset], mood: &Mood, tags: &[String]) -> Option<&'a Asset> {
        let mood_tags = &mood.tags;

        // Filter assets that match mood tags AND requested tags
        let candidates: Vec<&Asset> = assets
            .iter()
            .filter(|asset| {
                let asset_tags = asset.get_tags();

                // Check if asset has at least one tag from mood (or if mood has no tags)
                let matches_mood =
                    mood_tags.is_empty() || mood_tags.iter().any(|t| asset_tags.contains(t));

                // Check if asset has ALL requested tags
                let matches_request = tags.iter().all(|t| asset_tags.contains(t));

                matches_mood && matches_request
            })
            .collect();

        if candidates.is_empty() {
            // Fallback: Try matching just the requested tags if mood strictness allows (optional)
            // For now, let's just return None if no match found with mood constraint
            return None;
        }

        let mut rng = rand::rng();
        candidates.choose(&mut rng).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::types::ImageAsset;
    use std::path::PathBuf;

    fn create_test_registry() -> AssetRegistry {
        let mut registry = AssetRegistry::new();

        registry.add(Asset::Image(ImageAsset {
            path: PathBuf::from("img1.jpg"),
            tags: vec!["nature".to_string(), "calm".to_string()],
            width: 100,
            height: 100,
        }));

        registry.add(Asset::Image(ImageAsset {
            path: PathBuf::from("img2.jpg"),
            tags: vec!["city".to_string(), "busy".to_string()],
            width: 100,
            height: 100,
        }));

        registry.add(Asset::Image(ImageAsset {
            path: PathBuf::from("img3.jpg"),
            tags: vec!["nature".to_string(), "busy".to_string()],
            width: 100,
            height: 100,
        }));

        registry
    }

    #[test]
    fn test_select_image_mood_filtering() {
        let registry = create_test_registry();
        let selector = AssetSelector::new(&registry);

        let mood = Mood {
            name: "Relaxed".to_string(),
            description: "".to_string(),
            tags: vec!["calm".to_string()],
            prompt: None,
        };

        // Should only match img1 (nature, calm)
        let asset = selector.select_image(&mood, &[]);
        assert!(asset.is_some());
        assert_eq!(
            asset.unwrap().get_path().unwrap().to_str().unwrap(),
            "img1.jpg"
        );
    }

    #[test]
    fn test_select_image_tag_filtering() {
        let registry = create_test_registry();
        let selector = AssetSelector::new(&registry);

        let mood = Mood {
            name: "Any".to_string(),
            description: "".to_string(),
            tags: vec![], // No mood tags = allow all
            prompt: None,
        };

        // Request "busy" -> matches img2 and img3
        let asset = selector.select_image(&mood, &["busy".to_string()]);
        assert!(asset.is_some());
        let path = asset.unwrap().get_path().unwrap().to_str().unwrap();
        assert!(path == "img2.jpg" || path == "img3.jpg");
    }

    #[test]
    fn test_select_image_combined_filtering() {
        let registry = create_test_registry();
        let selector = AssetSelector::new(&registry);

        let mood = Mood {
            name: "Nature".to_string(),
            description: "".to_string(),
            tags: vec!["nature".to_string()],
            prompt: None,
        };

        // Mood "nature" (img1, img3) AND Request "busy" (img2, img3) -> Intersection is img3
        let asset = selector.select_image(&mood, &["busy".to_string()]);
        assert!(asset.is_some());
        assert_eq!(
            asset.unwrap().get_path().unwrap().to_str().unwrap(),
            "img3.jpg"
        );
    }

    #[test]
    fn test_select_no_match() {
        let registry = create_test_registry();
        let selector = AssetSelector::new(&registry);

        let mood = Mood {
            name: "Nature".to_string(),
            description: "".to_string(),
            tags: vec!["nature".to_string()],
            prompt: None,
        };

        // Mood "nature" AND Request "city" -> No match
        let asset = selector.select_image(&mood, &["city".to_string()]);
        assert!(asset.is_none());
    }

    #[test]
    fn test_select_video() {
        let mut registry = AssetRegistry::new();
        registry.add(Asset::Video(crate::assets::types::VideoAsset {
            path: PathBuf::from("vid1.mp4"),
            tags: vec!["action".to_string()],
            duration: None,
            width: 1920,
            height: 1080,
        }));

        let selector = AssetSelector::new(&registry);
        let mood = Mood {
            name: "Action".to_string(),
            description: "".to_string(),
            tags: vec!["action".to_string()],
            prompt: None,
        };

        let asset = selector.select_video(&mood, &[]);
        assert!(asset.is_some());
        if let Asset::Video(v) = asset.unwrap() {
            assert_eq!(v.path.to_str().unwrap(), "vid1.mp4");
        } else {
            panic!("Expected VideoAsset");
        }
    }

    #[test]
    fn test_select_audio() {
        let mut registry = AssetRegistry::new();
        registry.add(Asset::Audio(crate::assets::types::AudioAsset {
            path: PathBuf::from("audio1.mp3"),
            tags: vec!["ambient".to_string()],
            duration: None,
        }));

        let selector = AssetSelector::new(&registry);
        let mood = Mood {
            name: "Ambient".to_string(),
            description: "".to_string(),
            tags: vec!["ambient".to_string()],
            prompt: None,
        };

        let asset = selector.select_audio(&mood, &[]);
        assert!(asset.is_some());
        if let Asset::Audio(a) = asset.unwrap() {
            assert_eq!(a.path.to_str().unwrap(), "audio1.mp3");
        } else {
            panic!("Expected AudioAsset");
        }
    }

    #[test]
    fn test_select_hypno() {
        let mut registry = AssetRegistry::new();
        registry.add(Asset::Hypno(crate::assets::types::HypnoAsset {
            path: PathBuf::from("hypno1.gif"),
            tags: vec!["spiral".to_string()],
            is_animated: true,
        }));

        let selector = AssetSelector::new(&registry);
        let mood = Mood {
            name: "Trance".to_string(),
            description: "".to_string(),
            tags: vec!["spiral".to_string()],
            prompt: None,
        };

        let asset = selector.select_hypno(&mood, &[]);
        assert!(asset.is_some());
        if let Asset::Hypno(h) = asset.unwrap() {
            assert_eq!(h.path.to_str().unwrap(), "hypno1.gif");
        } else {
            panic!("Expected HypnoAsset");
        }
    }

    #[test]
    fn test_select_wallpaper() {
        let mut registry = AssetRegistry::new();
        registry.add(Asset::Wallpaper(crate::assets::types::WallpaperAsset {
            path: PathBuf::from("wall1.jpg"),
            tags: vec!["scenic".to_string()],
        }));

        let selector = AssetSelector::new(&registry);
        let mood = Mood {
            name: "Scenic".to_string(),
            description: "".to_string(),
            tags: vec!["scenic".to_string()],
            prompt: None,
        };

        let asset = selector.select_wallpaper(&mood, &[]);
        assert!(asset.is_some());
        if let Asset::Wallpaper(w) = asset.unwrap() {
            assert_eq!(w.path.to_str().unwrap(), "wall1.jpg");
        } else {
            panic!("Expected WallpaperAsset");
        }
    }
}
