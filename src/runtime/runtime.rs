use crate::gui::window_manager::GuiController;
use crate::permissions::PermissionChecker;
use crate::sdk;
use crate::sdk::{
    audio::goon_audio, hypno::goon_hypno, image::goon_image, prompt::goon_prompt,
    system::goon_system, video::goon_video, wallpaper::goon_wallpaper, website::goon_website,
};
use crate::typescript::TypeScriptCompiler;
use anyhow::Result;
use deno_core::{JsRuntime, ModuleSpecifier, RuntimeOptions};
use std::rc::Rc;

pub struct GoonRuntime {
    js_runtime: JsRuntime,
}

impl GoonRuntime {
    pub fn new(permissions: PermissionChecker, gui_controller: GuiController) -> Self {
        let mut js_runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![
                goon_system::init(),
                goon_image::init(),
                goon_video::init(),
                goon_audio::init(),
                goon_hypno::init(),
                goon_wallpaper::init(),
                goon_prompt::init(),
                goon_website::init(),
            ],
            ..Default::default()
        });

        // Store permissions in OpState
        {
            let op_state = js_runtime.op_state();
            let mut op_state = op_state.borrow_mut();
            op_state.put(permissions);
            op_state.put(gui_controller);
        }

        // Compile and load SDK bridge code
        let compiler = TypeScriptCompiler::new();
        let sources = sdk::get_all_typescript_sources();

        for source in sources {
            match compiler.compile(source) {
                Ok(js_code) => {
                    let _ = js_runtime.execute_script("sdk_bridge.js", js_code);
                }
                Err(e) => {
                    eprintln!("Failed to compile SDK bridge code: {}", e);
                }
            }
        }

        Self { js_runtime }
    }

    pub async fn execute_script(&mut self, code: &str) -> Result<()> {
        // We wrap the code in an async IIFE to support top-level await if needed,
        // but module loading handles top-level await natively.
        // Using load_main_es_module_from_code is better for modern JS.

        let main_module = deno_core::resolve_path("main.js", &std::env::current_dir()?)?;
        let mod_id = self
            .js_runtime
            .load_main_es_module_from_code(&main_module, code.to_string())
            .await?;

        let result = self.js_runtime.mod_evaluate(mod_id);
        self.js_runtime.run_event_loop(Default::default()).await?;
        result.await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{Permission, PermissionChecker, PermissionSet};

    #[tokio::test]
    async fn test_runtime_execution() {
        let mut set = PermissionSet::new();
        set.add(Permission::Image);
        let permissions = PermissionChecker::new(set);

        // Mock GuiController
        let event_loop =
            winit::event_loop::EventLoop::<crate::gui::event_loop::GuiCommand>::with_user_event()
                .build()
                .unwrap();
        let proxy = event_loop.create_proxy();
        let gui_controller = GuiController::new(proxy);

        let mut runtime = GoonRuntime::new(permissions, gui_controller);

        let code = r#"
            goon.system.log("Hello from JS");
            // const handle = await goon.image.show("test.png", {}); // This would fail without real window manager
            // goon.system.log("Image handle: " + handle);
        "#;

        let result = runtime.execute_script(code).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_permission_denied() {
        let set = PermissionSet::new(); // No permissions
        let permissions = PermissionChecker::new(set);

        // Mock GuiController
        let event_loop =
            winit::event_loop::EventLoop::<crate::gui::event_loop::GuiCommand>::with_user_event()
                .build()
                .unwrap();
        let proxy = event_loop.create_proxy();
        let gui_controller = GuiController::new(proxy);

        let mut runtime = GoonRuntime::new(permissions, gui_controller);

        let code = r#"
            await goon.image.show("test.png", {});
        "#;

        let result = runtime.execute_script(code).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Permission denied"));
    }
}
