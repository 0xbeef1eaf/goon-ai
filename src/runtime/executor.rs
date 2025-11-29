use crate::gui::window_manager::GuiController;
use crate::permissions::PermissionChecker;
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
        permissions: PermissionChecker,
        gui_controller: GuiController,
    ) -> Result<()> {
        let js_code = self.compiler.compile(ts_code)?;
        let mut runtime = GoonRuntime::new(permissions, gui_controller);
        runtime.execute_script(&js_code).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{PermissionChecker, PermissionSet};

    #[tokio::test]
    async fn test_executor_run() {
        let executor = Executor::new();
        let set = PermissionSet::new();
        let permissions = PermissionChecker::new(set);

        let event_loop =
            winit::event_loop::EventLoop::<crate::gui::event_loop::GuiCommand>::with_user_event()
                .build()
                .unwrap();
        let proxy = event_loop.create_proxy();
        let gui_controller = GuiController::new(proxy);

        let code = r#"
            goon.system.log("Executor test");
        "#;
        let result = executor.execute(code, permissions, gui_controller).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_executor_compile_error() {
        let executor = Executor::new();
        let set = PermissionSet::new();
        let permissions = PermissionChecker::new(set);

        let event_loop =
            winit::event_loop::EventLoop::<crate::gui::event_loop::GuiCommand>::with_user_event()
                .build()
                .unwrap();
        let proxy = event_loop.create_proxy();
        let gui_controller = GuiController::new(proxy);

        let code = "const x: number = ;"; // Invalid syntax
        let result = executor.execute(code, permissions, gui_controller).await;
        assert!(result.is_err());
    }
}
