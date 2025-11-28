use crate::permissions::PermissionChecker;
use crate::runtime::GoonRuntime;
use crate::typescript::TypeScriptCompiler;
use anyhow::Result;

pub struct Executor {
    compiler: TypeScriptCompiler,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            compiler: TypeScriptCompiler::new(),
        }
    }

    pub async fn execute(&self, ts_code: &str, permissions: PermissionChecker) -> Result<()> {
        let js_code = self.compiler.compile(ts_code)?;
        let mut runtime = GoonRuntime::new(permissions);
        runtime.execute_script(&js_code).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{PermissionSet, PermissionChecker};

    #[tokio::test]
    async fn test_executor_run() {
        let executor = Executor::new();
        let set = PermissionSet::new();
        let permissions = PermissionChecker::new(set);
        let code = r#"
            goon.system.log("Executor test");
        "#;
        let result = executor.execute(code, permissions).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_executor_compile_error() {
        let executor = Executor::new();
        let set = PermissionSet::new();
        let permissions = PermissionChecker::new(set);
        let code = "const x: number = ;"; // Invalid syntax
        let result = executor.execute(code, permissions).await;
        assert!(result.is_err());
    }
}
