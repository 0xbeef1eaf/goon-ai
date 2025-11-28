use crate::assets::registry::AssetRegistry;
use crate::assets::types::Asset;
use crate::config::pack::Mood;
use rand::seq::SliceRandom;

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

        let mut rng = rand::thread_rng();
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
        }));

        registry.add(Asset::Image(ImageAsset {
            path: PathBuf::from("img2.jpg"),
            tags: vec!["city".to_string(), "busy".to_string()],
        }));

        registry.add(Asset::Image(ImageAsset {
            path: PathBuf::from("img3.jpg"),
            tags: vec!["nature".to_string(), "busy".to_string()],
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
        };

        // Should only match img1 (nature, calm)
        let asset = selector.select_image(&mood, &[]);
        assert!(asset.is_some());
        assert_eq!(asset.unwrap().get_path().to_str().unwrap(), "img1.jpg");
    }

    #[test]
    fn test_select_image_tag_filtering() {
        let registry = create_test_registry();
        let selector = AssetSelector::new(&registry);

        let mood = Mood {
            name: "Any".to_string(),
            description: "".to_string(),
            tags: vec![], // No mood tags = allow all
        };

        // Request "busy" -> matches img2 and img3
        let asset = selector.select_image(&mood, &["busy".to_string()]);
        assert!(asset.is_some());
        let path = asset.unwrap().get_path().to_str().unwrap();
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
        };

        // Mood "nature" (img1, img3) AND Request "busy" (img2, img3) -> Intersection is img3
        let asset = selector.select_image(&mood, &["busy".to_string()]);
        assert!(asset.is_some());
        assert_eq!(asset.unwrap().get_path().to_str().unwrap(), "img3.jpg");
    }

    #[test]
    fn test_select_no_match() {
        let registry = create_test_registry();
        let selector = AssetSelector::new(&registry);

        let mood = Mood {
            name: "Nature".to_string(),
            description: "".to_string(),
            tags: vec!["nature".to_string()],
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
        }));

        let selector = AssetSelector::new(&registry);
        let mood = Mood {
            name: "Action".to_string(),
            description: "".to_string(),
            tags: vec!["action".to_string()],
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
        };

        let asset = selector.select_audio(&mood, &[]);
        assert!(asset.is_some());
        if let Asset::Audio(a) = asset.unwrap() {
            assert_eq!(a.path.to_str().unwrap(), "audio1.mp3");
        } else {
            panic!("Expected AudioAsset");
        }
    }
}
