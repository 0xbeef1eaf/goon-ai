use crate::assets::registry::AssetRegistry;
use crate::config::pack::Mood;
use crate::gui::window_manager::GuiInterface;
use crate::permissions::PermissionChecker;
use crate::runtime::GoonRuntime;
use crate::typescript::TypeScriptCompiler;
use anyhow::Result;

use std::sync::Arc;

pub struct Executor {
    compiler: TypeScriptCompiler,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn new() -> Self {
        Self {
            compiler: TypeScriptCompiler::new(),
        }
    }

    pub async fn execute(
        &self,
        ts_code: &str,
        context: crate::runtime::runtime::RuntimeContext,
    ) -> Result<()> {
        let js_code = self.compiler.compile(ts_code)?;
        let mut runtime = GoonRuntime::new(context);
        runtime.execute_script(&js_code).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::content::ContentConstructor;
    use crate::gui::window_manager::{GuiInterface, WindowHandle, WindowOptions};
    use crate::permissions::{PermissionChecker, PermissionSet};

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
    async fn test_executor_run() {
        let executor = Executor::new();
        let set = PermissionSet::new();
        let permissions = PermissionChecker::new(set);

        let gui_controller = Arc::new(MockGuiController);
        let registry = Arc::new(AssetRegistry::new());
        let mood = Mood {
            name: "Test".to_string(),
            description: "".to_string(),
            tags: vec![],
        };

        let context = crate::runtime::runtime::RuntimeContext {
            permissions,
            gui_controller,
            registry,
            mood,
            max_audio_concurrent: 10,
            max_video_concurrent: 3,
        };

        let code = r#"
            goon.system.log("Executor test");
        "#;
        let result = executor.execute(code, context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_executor_compile_error() {
        let executor = Executor::new();
        let set = PermissionSet::new();
        let permissions = PermissionChecker::new(set);

        let gui_controller = Arc::new(MockGuiController);
        let registry = Arc::new(AssetRegistry::new());
        let mood = Mood {
            name: "Test".to_string(),
            description: "".to_string(),
            tags: vec![],
        };

        let context = crate::runtime::runtime::RuntimeContext {
            permissions,
            gui_controller,
            registry,
            mood,
            max_audio_concurrent: 10,
            max_video_concurrent: 3,
        };

        let code = "const x: number = ;"; // Invalid syntax
        let result = executor.execute(code, context).await;
        assert!(result.is_err());
    }
}
