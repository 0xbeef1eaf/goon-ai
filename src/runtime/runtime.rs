use crate::assets::registry::AssetRegistry;
use crate::config::pack::Mood;
use crate::gui::WindowSpawnerHandle;
use crate::media::audio::manager::AudioManager;
use crate::permissions::PermissionChecker;
use crate::sdk;
use crate::sdk::{
    audio::goon_audio, hypno::goon_hypno, image::goon_image, pack::goon_pack, system::goon_system,
    video::goon_video, wallpaper::goon_wallpaper, website::goon_website,
    write_lines::goon_write_lines,
};
use crate::typescript::TypeScriptCompiler;
use anyhow::Result;
use deno_core::{JsRuntime, RuntimeOptions};
use rodio::{OutputStream, OutputStreamBuilder, mixer::Mixer};
use std::sync::{Arc, Mutex};

pub struct RuntimeContext {
    pub permissions: PermissionChecker,
    pub window_spawner: WindowSpawnerHandle,
    pub registry: Arc<AssetRegistry>,
    pub mood: Mood,
    pub max_audio_concurrent: usize,
}

pub struct GoonRuntime {
    pub js_runtime: JsRuntime,
    _audio_stream: Option<OutputStream>,
}

impl GoonRuntime {
    pub fn new(context: RuntimeContext) -> Self {
        let (audio_stream, mixer) = match OutputStreamBuilder::open_default_stream() {
            Ok(s) => {
                let mixer = s.mixer().clone();
                (Some(s), Some(mixer))
            }
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
                goon_write_lines::init(),
                goon_website::init(),
            ],
            ..Default::default()
        });

        // Store permissions in OpState
        {
            let op_state = js_runtime.op_state();
            let mut op_state = op_state.borrow_mut();
            op_state.put(context.permissions);
            op_state.put(context.window_spawner);
            op_state.put(context.registry);
            op_state.put(context.mood);

            if let Some(m) = mixer {
                let audio_manager = Arc::new(Mutex::new(AudioManager::new(
                    m,
                    context.max_audio_concurrent,
                )));
                op_state.put(audio_manager);
            }
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
    use crate::gui::WindowSpawner;
    use crate::permissions::{Permission, PermissionChecker, PermissionSet};

    fn create_test_context() -> (RuntimeContext, crate::gui::WindowSpawner) {
        let mut set = PermissionSet::new();
        set.add(Permission::Image);
        let permissions = PermissionChecker::new(set);

        let (window_handle, window_spawner) = WindowSpawner::create();
        let registry = Arc::new(AssetRegistry::new());
        let mood = Mood {
            name: "Test".to_string(),
            description: "".to_string(),
            tags: vec![],
            prompt: None,
        };
        let context = RuntimeContext {
            permissions,
            window_spawner: window_handle,
            registry,
            mood,
            max_audio_concurrent: 10,
        };
        (context, window_spawner)
    }

    #[tokio::test]
    async fn test_runtime_execution() {
        let (context, _spawner) = create_test_context();
        let mut runtime = GoonRuntime::new(context);

        let code = r#"
            goon.pack.getCurrentMood();
            // await goon.image.show({ tags: ["test"], duration: 1000 });
        "#;

        let result = runtime.execute_script(code).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bigint_duration() {
        let (context, _spawner) = create_test_context();
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

        let (window_handle, _spawner) = WindowSpawner::create();
        let registry = Arc::new(AssetRegistry::new());
        let mood = Mood {
            name: "Test".to_string(),
            description: "".to_string(),
            tags: vec![],
            prompt: None,
        };
        let context = RuntimeContext {
            permissions,
            window_spawner: window_handle,
            registry,
            mood,
            max_audio_concurrent: 10,
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

        let (window_handle, _spawner) = WindowSpawner::create();
        let registry = Arc::new(AssetRegistry::new());
        let mood = Mood {
            name: "TestMood".to_string(),
            description: "A test mood".to_string(),
            tags: vec!["tag1".to_string()],
            prompt: None,
        };
        let context = RuntimeContext {
            permissions,
            window_spawner: window_handle,
            registry,
            mood,
            max_audio_concurrent: 10,
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
