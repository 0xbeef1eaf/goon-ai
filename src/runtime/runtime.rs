use deno_core::{JsRuntime, RuntimeOptions, ModuleSpecifier};
use std::rc::Rc;
use anyhow::Result;
use crate::permissions::Permissions;
use crate::sdk::{
    image::goon_image,
    video::goon_video,
    audio::goon_audio,
    hypno::goon_hypno,
    wallpaper::goon_wallpaper,
    prompt::goon_prompt,
    website::goon_website,
    system::goon_system,
};
use crate::typescript::TypeScriptCompiler;
use crate::sdk;

pub struct GoonRuntime {
    js_runtime: JsRuntime,
}

impl GoonRuntime {
    pub fn new(permissions: Permissions) -> Self {
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
        }

        // Compile and load SDK bridge code
        let compiler = TypeScriptCompiler::new();
        let sources = sdk::get_all_typescript_sources();
        
        for source in sources {
            match compiler.compile(source) {
                Ok(js_code) => {
                    let _ = js_runtime.execute_script("sdk_bridge.js", js_code);
                },
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
        let mod_id = self.js_runtime.load_main_es_module_from_code(
            &main_module,
            code.to_string(),
        ).await?;
        
        let result = self.js_runtime.mod_evaluate(mod_id);
        self.js_runtime.run_event_loop(Default::default()).await?;
        result.await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::Permissions;

    #[tokio::test]
    async fn test_runtime_execution() {
        let permissions = Permissions { allowed: vec!["all".to_string()] };
        let mut runtime = GoonRuntime::new(permissions);
        
        let code = r#"
            goon.system.log("Hello from JS");
            const handle = await goon.image.show("test.png", {});
            goon.system.log("Image handle: " + handle);
        "#;
        
        let result = runtime.execute_script(code).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_permission_denied() {
        let permissions = Permissions { allowed: vec![] }; // No permissions
        let mut runtime = GoonRuntime::new(permissions);
        
        let code = r#"
            await goon.image.show("test.png", {});
        "#;
        
        let result = runtime.execute_script(code).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Permission denied"));
    }
}
