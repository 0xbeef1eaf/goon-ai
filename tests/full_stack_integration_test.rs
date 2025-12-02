use goon_ai::assets::loader::AssetLoader;
use goon_ai::assets::selector::AssetSelector;
use goon_ai::assets::types::Asset;
use goon_ai::config::pack::{Asset as ConfigAsset, Assets, Mood, PackConfig, PackMeta};
use goon_ai::gui::WindowSpawner;
use goon_ai::permissions::{Permission, PermissionChecker, PermissionResolver, PermissionSet};
use goon_ai::runtime::GoonRuntime;
use goon_ai::runtime::runtime::RuntimeContext;

#[tokio::test]
async fn test_asset_loading_to_permission_check() {
    // 1. Setup Configuration
    // Pack: Requests Image and Video permissions. Contains Image and Video assets.
    let pack_config = PackConfig {
        meta: PackMeta {
            name: "TestPack".to_string(),
            version: "1.0.0".to_string(),
            permissions: vec![Permission::Image, Permission::Video],
        },
        moods: vec![Mood {
            name: "Default".to_string(),
            description: "Default mood".to_string(),
            tags: vec!["default".to_string()],
            prompt: None,
        }],
        assets: Assets {
            image: Some(vec![ConfigAsset {
                path: "image/beach.jpg".to_string(),
                tags: vec!["default".to_string()],
            }]),
            video: Some(vec![ConfigAsset {
                path: "video/test-bunny.mp4".to_string(),
                tags: vec!["default".to_string()],
            }]),
            audio: None,
            hypno: None,
            wallpaper: None,
        },
        websites: None,
        prompts: None,
    };

    // User: Grants ONLY Image permission.
    let mut user_perms = PermissionSet::new();
    user_perms.add(Permission::Image);

    // 2. Load Assets
    // This creates the registry with paths relative to "packs/TestPack"
    let registry = AssetLoader::load(&pack_config, "TestPack").expect("Failed to load assets");

    // 3. Resolve Permissions
    let pack_perms: PermissionSet = pack_config.meta.permissions.clone().into();
    let active_perms = PermissionResolver::resolve(&pack_perms, &user_perms);

    // Verify resolution logic
    assert!(active_perms.contains(Permission::Image));
    assert!(!active_perms.contains(Permission::Video));

    let checker = PermissionChecker::new(active_perms);

    // 4. Select Assets (Simulating System Logic)
    let selector = AssetSelector::new(&registry);
    let mood = &pack_config.moods[0];

    // Select Image
    let image_asset = selector
        .select_image(mood, &["default".to_string()])
        .expect("Should find image asset");

    let _image_path = if let Asset::Image(img) = image_asset {
        img.path.to_str().unwrap().to_string()
    } else {
        panic!("Expected ImageAsset");
    };

    // Select Video
    let video_asset = selector
        .select_video(mood, &["default".to_string()])
        .expect("Should find video asset");

    let _video_path = if let Asset::Video(vid) = video_asset {
        vid.path.to_str().unwrap().to_string()
    } else {
        panic!("Expected VideoAsset");
    };

    // 5. Runtime Execution
    // We use the checker derived from the resolution step.

    // Use WindowSpawner
    let (window_spawner, _spawner) = WindowSpawner::create();
    let registry_arc = std::sync::Arc::new(registry);
    let mood_clone = mood.clone();

    let context = RuntimeContext {
        permissions: checker.clone(),
        window_spawner: window_spawner.clone(),
        registry: registry_arc.clone(),
        mood: mood_clone.clone(),
        max_audio_concurrent: 10,
        max_video_concurrent: 3,
    };

    let mut runtime = GoonRuntime::new(context);

    // A. Attempt to show the selected Image (Permission Granted)
    // We inject the selected path into the JS code, simulating the LLM using a path provided by the system (or known to it).
    let code_image = r#"
        (async () => {
            try {
                await goon.image.show({ tags: ["default"] });
            } catch (e) {
                throw new Error("Image show failed: " + e.message);
            }
        })()
    "#;

    let result = runtime.execute_script(&code_image).await;
    assert!(
        result.is_ok(),
        "Allowed image operation failed: {:?}",
        result.err()
    );

    // B. Attempt to show the selected Video (Permission Denied)
    // Create a new runtime to avoid module name collision
    let context2 = RuntimeContext {
        permissions: checker.clone(),
        window_spawner: window_spawner.clone(),
        registry: registry_arc.clone(),
        mood: mood_clone.clone(),
        max_audio_concurrent: 10,
        max_video_concurrent: 3,
    };

    let mut runtime2 = GoonRuntime::new(context2);

    let code_video = r#"
        (async () => {
            await goon.video.play({ tags: ["default"] });
        })()
    "#;

    let result = runtime2.execute_script(&code_video).await;
    assert!(
        result.is_err(),
        "Denied video operation succeeded unexpectedly"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Permission denied"),
        "Error should be permission denied, got: {}",
        err_msg
    );
}
