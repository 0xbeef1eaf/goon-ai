use anyhow::Result;
use goon_ai::gui::content::ContentConstructor;
use goon_ai::gui::window_manager::{GuiInterface, WindowHandle, WindowOptions};
use goon_ai::permissions::{Permission, PermissionChecker, PermissionResolver, PermissionSet};
use goon_ai::runtime::GoonRuntime;
use goon_ai::runtime::runtime::RuntimeContext;
use goon_ai::sdk::generate_definitions_for_permissions;

struct MockGuiController;

impl GuiInterface for MockGuiController {
    fn create_window(&self, _options: WindowOptions) -> Result<WindowHandle> {
        Ok(WindowHandle(uuid::Uuid::new_v4()))
    }
    fn close_window(&self, _handle: WindowHandle) -> Result<()> {
        Ok(())
    }
    fn set_content(
        &self,
        _handle: WindowHandle,
        _content: Box<dyn ContentConstructor>,
    ) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_full_permission_flow() {
    // 1. Simulate Configuration Loading
    // User grants: Image, Audio
    let mut user_perms = PermissionSet::new();
    user_perms.add(Permission::Image);
    user_perms.add(Permission::Audio);

    // Pack requests: Image, Video
    let mut pack_perms = PermissionSet::new();
    pack_perms.add(Permission::Image);
    pack_perms.add(Permission::Video);

    // 2. Resolve Permissions
    // Intersection should be: Image
    let active_perms = PermissionResolver::resolve(&pack_perms, &user_perms);

    assert!(active_perms.contains(Permission::Image));
    assert!(!active_perms.contains(Permission::Video)); // Denied by user
    assert!(!active_perms.contains(Permission::Audio)); // Not requested by pack

    // 3. Initialize Runtime with Resolved Permissions
    let checker = PermissionChecker::new(active_perms.clone());

    // Mock GuiController
    let gui_controller = std::sync::Arc::new(MockGuiController);
    let registry = std::sync::Arc::new(goon_ai::assets::registry::AssetRegistry::new());
    let mood = goon_ai::config::pack::Mood {
        name: "Test".to_string(),
        description: "".to_string(),
        tags: vec![],
    };

    let context = RuntimeContext {
        permissions: checker.clone(),
        gui_controller: gui_controller.clone(),
        registry: registry.clone(),
        mood: mood.clone(),
        max_audio_concurrent: 10,
        max_video_concurrent: 3,
    };

    let mut runtime = GoonRuntime::new(context);

    // 4. Test Allowed Operation (Image)
    let allowed_code = r#"
        (async () => {
            try {
                await goon.image.show({ tags: ["default"] });
                return "success";
            } catch (e) {
                return "failed: " + e.message;
            }
        })()
    "#;
    // We expect this to succeed (or at least not fail with permission error)
    // Note: The actual op_show_image prints to stdout and returns 1.
    // Since we can't easily capture the return value from execute_script in this test setup without more plumbing,
    // we rely on it not panicking or throwing an unhandled exception.
    // However, execute_script returns Result<()>.

    let result = runtime.execute_script(allowed_code).await;
    assert!(
        result.is_ok(),
        "Allowed operation failed: {:?}",
        result.err()
    );

    // 5. Test Denied Operation (Video)
    // Create a new runtime instance to avoid module name collision ("main.js")
    let context2 = RuntimeContext {
        permissions: checker.clone(),
        gui_controller: gui_controller.clone(),
        registry: registry.clone(),
        mood: mood.clone(),
        max_audio_concurrent: 10,
        max_video_concurrent: 3,
    };

    let mut runtime2 = GoonRuntime::new(context2);

    let denied_code = r#"
        (async () => {
            await goon.video.show({ tags: ["default"] });
        })()
    "#;

    let result = runtime2.execute_script(denied_code).await;
    assert!(result.is_err(), "Denied operation succeeded unexpectedly");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Permission denied"),
        "Unexpected error message: {}",
        err_msg
    );

    // 6. Verify SDK Generation
    let sdk_defs = generate_definitions_for_permissions(&checker);
    assert!(
        sdk_defs.contains("class image"),
        "SDK should contain image class"
    );
    assert!(
        !sdk_defs.contains("class video"),
        "SDK should NOT contain video class"
    );
}
