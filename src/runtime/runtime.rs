use crate::assets::registry::AssetRegistry;
use crate::config::pack::Mood;
use crate::gui::window_manager::GuiInterface;
use crate::media::audio::manager::AudioManager;
use crate::media::video::manager::VideoManager;
use crate::permissions::PermissionChecker;
use crate::sdk;
use crate::sdk::{
    audio::goon_audio, hypno::goon_hypno, image::goon_image, pack::goon_pack, prompt::goon_prompt,
    system::goon_system, video::goon_video, wallpaper::goon_wallpaper, website::goon_website,
};
use crate::typescript::TypeScriptCompiler;
use anyhow::Result;
use deno_core::{JsRuntime, ModuleSpecifier, RuntimeOptions};
use rodio::OutputStream;
use std::sync::{Arc, Mutex};

pub struct RuntimeContext {
    pub permissions: PermissionChecker,
    pub gui_controller: Arc<dyn GuiInterface>,
    pub registry: Arc<AssetRegistry>,
    pub mood: Mood,
    pub max_audio_concurrent: usize,
    pub max_video_concurrent: usize,
}

pub struct GoonRuntime {
    js_runtime: JsRuntime,
    _audio_stream: Option<OutputStream>,
}

impl GoonRuntime {
    pub fn new(context: RuntimeContext) -> Self {
        let (audio_stream, stream_handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                eprintln!("Failed to initialize audio device: {}", e);
                (None, None)
            }
        };

        let mut js_runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![
                goon_system::init(),
                goon_pack::init(),
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
            op_state.put(context.permissions);
            op_state.put(context.gui_controller);
            op_state.put(context.registry);
            op_state.put(context.mood);

            if let Some(handle) = stream_handle {
                let audio_manager = Arc::new(Mutex::new(AudioManager::new(
                    handle,
                    context.max_audio_concurrent,
                )));
                op_state.put(audio_manager);
            }

            let video_manager = Arc::new(tokio::sync::Mutex::new(VideoManager::new(
                context.max_video_concurrent,
            )));
            op_state.put(video_manager);
        }

        // Compile and load SDK bridge code
        let compiler = TypeScriptCompiler::new();
        let sources = sdk::get_all_typescript_sources();

        for source in sources {
            match compiler.compile(&source) {
                Ok(js_code) => {
                    if let Err(e) = js_runtime.execute_script("sdk_bridge.js", js_code) {
                        eprintln!("Failed to execute SDK bridge code: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to compile SDK bridge code: {}", e);
                }
            }
        }

        Self {
            js_runtime,
            _audio_stream: audio_stream,
        }
    }

    pub async fn execute_script(&mut self, code: &str) -> Result<()> {
        // We wrap the code in an async IIFE to support top-level await
        // and ensure we handle the promise result.
        // We also need to strip import statements as we are running as a script.

        // Simple strip of import lines (this is a heuristic)
        let code_lines: Vec<&str> = code
            .lines()
            .filter(|line| !line.trim().starts_with("import "))
            .collect();
        let clean_code = code_lines.join("\n");

        let wrapped_code = format!("(async () => {{ {} }})()", clean_code);

        // execute_script returns the result of the expression
        let _promise = self
            .js_runtime
            .execute_script("user_script.js", wrapped_code)?;

        // Run event loop to handle any pending ops
        self.js_runtime.run_event_loop(Default::default()).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::content::ContentConstructor;
    use crate::gui::window_manager::{GuiInterface, WindowHandle, WindowOptions};
    use crate::permissions::{Permission, PermissionChecker, PermissionSet};

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
    async fn test_runtime_execution() {
        let mut set = PermissionSet::new();
        set.add(Permission::Image);
        let permissions = PermissionChecker::new(set);

        let gui_controller = Arc::new(MockGuiController);
        let registry = Arc::new(AssetRegistry::new());
        let mood = Mood {
            name: "Test".to_string(),
            description: "".to_string(),
            tags: vec![],
        };
        let context = RuntimeContext {
            permissions,
            gui_controller,
            registry,
            mood,
            max_audio_concurrent: 10,
            max_video_concurrent: 3,
        };
        let mut runtime = GoonRuntime::new(context);

        let code = r#"
            goon.system.log("Hello from JS");
            // const handle = await goon.image.show({ tags: ["test"] }); // This would fail without real window manager
            // goon.system.log("Image handle: " + handle);
        "#;

        let result = runtime.execute_script(code).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bigint_duration() {
        let mut set = PermissionSet::new();
        set.add(Permission::Image);
        let permissions = PermissionChecker::new(set);

        let gui_controller = Arc::new(MockGuiController);
        let registry = Arc::new(AssetRegistry::new());
        let mood = Mood {
            name: "Test".to_string(),
            description: "".to_string(),
            tags: vec![],
        };
        let context = RuntimeContext {
            permissions,
            gui_controller,
            registry,
            mood,
            max_audio_concurrent: 10,
            max_video_concurrent: 3,
        };
        let mut runtime = GoonRuntime::new(context);

        let code = r#"
            try {
                await goon.image.show({ tags: [], duration: 10n });
            } catch (e) {
                if (e.message.includes("No image found")) {
                    // This is expected as registry is empty
                } else {
                    throw e;
                }
            }
        "#;

        let result = runtime.execute_script(code).await;
        if let Err(e) = &result {
            eprintln!("Test failed: {}", e);
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_permission_denied() {
        let set = PermissionSet::new(); // No permissions
        let permissions = PermissionChecker::new(set);

        let gui_controller = Arc::new(MockGuiController);
        let registry = Arc::new(AssetRegistry::new());
        let mood = Mood {
            name: "Test".to_string(),
            description: "".to_string(),
            tags: vec![],
        };
        let context = RuntimeContext {
            permissions,
            gui_controller,
            registry,
            mood,
            max_audio_concurrent: 10,
            max_video_concurrent: 3,
        };
        let mut runtime = GoonRuntime::new(context);

        let code = r#"
            await goon.image.show({ tags: ["test"] });
        "#;

        let result = runtime.execute_script(code).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Permission denied"));
    }

    #[tokio::test]
    async fn test_pack_availability() {
        let mut set = PermissionSet::new();
        set.add(Permission::Image); // Just some permission
        let permissions = PermissionChecker::new(set);

        let gui_controller = Arc::new(MockGuiController);
        let registry = Arc::new(AssetRegistry::new());
        let mood = Mood {
            name: "TestMood".to_string(),
            description: "A test mood".to_string(),
            tags: vec!["tag1".to_string()],
        };
        let context = RuntimeContext {
            permissions,
            gui_controller,
            registry,
            mood,
            max_audio_concurrent: 10,
            max_video_concurrent: 3,
        };
        let mut runtime = GoonRuntime::new(context);

        let code = r#"
            const mood = await goon.pack.getCurrentMood();
            if (mood.name !== "TestMood") {
                throw new Error("Wrong mood name: " + mood.name);
            }
            if (mood.tags[0] !== "tag1") {
                throw new Error("Wrong mood tag: " + mood.tags[0]);
            }

            await goon.pack.setMood("NewMood");
            const newMood = await goon.pack.getCurrentMood();
            if (newMood.name !== "NewMood") {
                throw new Error("Failed to set mood: " + newMood.name);
            }
        "#;

        let result = runtime.execute_script(code).await;
        if let Err(e) = &result {
            eprintln!("Test failed: {}", e);
        }
        assert!(result.is_ok());
    }
}
