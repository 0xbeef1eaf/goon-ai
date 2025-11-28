use goon_ai::assets::loader::AssetLoader;
use goon_ai::assets::selector::AssetSelector;
use goon_ai::assets::types::Asset;
use goon_ai::config::pack::{Asset as ConfigAsset, Assets, Mood, PackConfig, PackMeta};
use goon_ai::permissions::Permission;

#[test]
fn test_asset_flow_from_config_to_selection() {
    // 1. Setup Config
    let pack_config = PackConfig {
        meta: PackMeta {
            name: "IntegrationPack".to_string(),
            version: "1.0.0".to_string(),
            permissions: vec![Permission::Image, Permission::Video],
        },
        moods: vec![
            Mood {
                name: "Happy".to_string(),
                description: "Joyful content".to_string(),
                tags: vec!["happy".to_string()],
            },
            Mood {
                name: "Sad".to_string(),
                description: "Melancholy content".to_string(),
                tags: vec!["sad".to_string()],
            },
        ],
        assets: Assets {
            image: Some(vec![
                ConfigAsset {
                    path: "happy_img.jpg".to_string(),
                    tags: vec!["happy".to_string(), "bright".to_string()],
                },
                ConfigAsset {
                    path: "sad_img.jpg".to_string(),
                    tags: vec!["sad".to_string(), "dark".to_string()],
                },
                ConfigAsset {
                    path: "neutral_img.jpg".to_string(),
                    tags: vec!["neutral".to_string()],
                },
            ]),
            video: Some(vec![ConfigAsset {
                path: "happy_vid.mp4".to_string(),
                tags: vec!["happy".to_string(), "dance".to_string()],
            }]),
            audio: None,
            hypno: None,
            wallpaper: None,
        },
    };

    // 2. Load Assets
    let registry =
        AssetLoader::load(&pack_config, "IntegrationPack").expect("Failed to load assets");

    assert_eq!(registry.images.len(), 3);
    assert_eq!(registry.videos.len(), 1);

    // 3. Select Assets
    let selector = AssetSelector::new(&registry);

    // Case A: Happy Mood, Request "bright" image
    let happy_mood = &pack_config.moods[0]; // Happy
    let asset = selector.select_image(happy_mood, &["bright".to_string()]);

    assert!(asset.is_some());
    if let Asset::Image(img) = asset.unwrap() {
        assert!(img.path.to_str().unwrap().contains("happy_img.jpg"));
    } else {
        panic!("Expected ImageAsset");
    }

    // Case B: Sad Mood, Request "happy" image (Should fail because mood filters for "sad")
    let sad_mood = &pack_config.moods[1]; // Sad
    let asset = selector.select_image(sad_mood, &["happy".to_string()]);
    assert!(asset.is_none());

    // Case C: Happy Mood, Request Video
    let asset = selector.select_video(happy_mood, &["dance".to_string()]);
    assert!(asset.is_some());
    if let Asset::Video(vid) = asset.unwrap() {
        assert!(vid.path.to_str().unwrap().contains("happy_vid.mp4"));
    } else {
        panic!("Expected VideoAsset");
    }
}
