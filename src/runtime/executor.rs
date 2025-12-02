use crate::config::pack::Mood;
use crate::runtime::GoonRuntime;
use crate::typescript::TypeScriptCompiler;
use anyhow::Result;

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
    use crate::assets::registry::AssetRegistry;
    use crate::gui::WindowSpawner;
    use crate::permissions::{PermissionChecker, PermissionSet};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_executor_run() {
        let executor = Executor::new();
        let set = PermissionSet::new();
        let permissions = PermissionChecker::new(set);

        let (window_handle, _spawner) = WindowSpawner::create();
        let registry = Arc::new(AssetRegistry::new());
        let mood = Mood {
            name: "Test".to_string(),
            description: "".to_string(),
            tags: vec![],
            prompt: None,
        };

        let context = crate::runtime::runtime::RuntimeContext {
            permissions,
            window_spawner: window_handle,
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

        let (window_handle, _spawner) = WindowSpawner::create();
        let registry = Arc::new(AssetRegistry::new());
        let mood = Mood {
            name: "Test".to_string(),
            description: "".to_string(),
            tags: vec![],
            prompt: None,
        };

        let context = crate::runtime::runtime::RuntimeContext {
            permissions,
            window_spawner: window_handle,
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
