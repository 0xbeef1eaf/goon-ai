use goon_ai::assets::loader::AssetLoader;
use goon_ai::assets::selector::AssetSelector;
use goon_ai::assets::types::Asset;
use goon_ai::config::pack::{Asset as ConfigAsset, Assets, Mood, PackConfig, PackMeta};
use goon_ai::permissions::{Permission, PermissionChecker, PermissionResolver, PermissionSet};
use goon_ai::runtime::GoonRuntime;

#[tokio::test]
async fn test_asset_loading_to_permission_check() {
    // 1. Setup Configuration
    // Pack: Requests Image and Video permissions. Contains Image and Video assets.
    let pack_config = PackConfig {
        meta: PackMeta {
            name: "FullStackPack".to_string(),
            version: "1.0.0".to_string(),
            permissions: vec![Permission::Image, Permission::Video],
        },
        moods: vec![Mood {
            name: "Default".to_string(),
            description: "Default mood".to_string(),
            tags: vec!["default".to_string()],
        }],
        assets: Assets {
            image: Some(vec![ConfigAsset {
                path: "img/test.jpg".to_string(),
                tags: vec!["default".to_string()],
            }]),
            video: Some(vec![ConfigAsset {
                path: "vid/test.mp4".to_string(),
                tags: vec!["default".to_string()],
            }]),
            audio: None,
            hypno: None,
            wallpaper: None,
        },
    };

    // User: Grants ONLY Image permission.
    let mut user_perms = PermissionSet::new();
    user_perms.add(Permission::Image);

    // 2. Load Assets
    // This creates the registry with paths relative to "packs/FullStackPack"
    let registry = AssetLoader::load(&pack_config, "FullStackPack").expect("Failed to load assets");

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

    let image_path = if let Asset::Image(img) = image_asset {
        img.path.to_str().unwrap().to_string()
    } else {
        panic!("Expected ImageAsset");
    };

    // Select Video
    let video_asset = selector
        .select_video(mood, &["default".to_string()])
        .expect("Should find video asset");

    let video_path = if let Asset::Video(vid) = video_asset {
        vid.path.to_str().unwrap().to_string()
    } else {
        panic!("Expected VideoAsset");
    };

    // 5. Runtime Execution
    // We use the checker derived from the resolution step.
    let mut runtime = GoonRuntime::new(checker.clone());

    // A. Attempt to show the selected Image (Permission Granted)
    // We inject the selected path into the JS code, simulating the LLM using a path provided by the system (or known to it).
    let code_image = format!(
        r#"
        (async () => {{
            try {{
                await goon.image.show("{}", {{}});
            }} catch (e) {{
                throw new Error("Image show failed: " + e.message);
            }}
        }})()
    "#,
        image_path.replace("\\", "/")
    ); // Handle Windows paths if necessary

    let result = runtime.execute_script(&code_image).await;
    assert!(
        result.is_ok(),
        "Allowed image operation failed: {:?}",
        result.err()
    );

    // B. Attempt to show the selected Video (Permission Denied)
    // Create a new runtime to avoid module name collision
    let mut runtime2 = GoonRuntime::new(checker.clone());

    let code_video = format!(
        r#"
        (async () => {{
            await goon.video.show("{}", {{}});
        }})()
    "#,
        video_path.replace("\\", "/")
    );

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
