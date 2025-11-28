use crate::typescript::TypeScriptCompiler;
use crate::runtime::GoonRuntime;
use crate::permissions::Permissions;
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

    pub async fn execute(&self, ts_code: &str, permissions: Permissions) -> Result<()> {
        let js_code = self.compiler.compile(ts_code)?;
        let mut runtime = GoonRuntime::new(permissions);
        runtime.execute_script(&js_code).await?;
        Ok(())
    }
}
