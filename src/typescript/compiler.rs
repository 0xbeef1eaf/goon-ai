use crate::typescript::error::CompilationError;
use std::sync::Arc;
use swc::Compiler;
use swc_common::{
    FileName, GLOBALS, Globals, SourceMap,
    errors::{ColorConfig, Handler},
};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{Syntax, TsSyntax};

pub struct TypeScriptCompiler {
    compiler: Arc<Compiler>,
    cm: Arc<SourceMap>,
}

impl Default for TypeScriptCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptCompiler {
    pub fn new() -> Self {
        let cm = Arc::new(SourceMap::default());
        let compiler = Arc::new(Compiler::new(cm.clone()));
        Self { compiler, cm }
    }

    pub fn compile(&self, source: &str) -> Result<String, CompilationError> {
        let globals = Globals::new();
        GLOBALS.set(&globals, || {
            let handler =
                Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(self.cm.clone()));

            let fm = self.cm.new_source_file(
                FileName::Custom("script.ts".into()).into(),
                source.to_string(),
            );

            let result = self.compiler.process_js_file(
                fm,
                &handler,
                &swc::config::Options {
                    config: swc::config::Config {
                        jsc: swc::config::JscConfig {
                            syntax: Some(Syntax::Typescript(TsSyntax {
                                tsx: false,
                                decorators: false,
                                dts: false,
                                no_early_errors: false,
                                disallow_ambiguous_jsx_like: false,
                            })),
                            target: Some(EsVersion::Es2020),
                            external_helpers: false.into(), // Inline helpers instead of importing
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );

            match result {
                Ok(output) => Ok(output.code),
                Err(e) => Err(CompilationError {
                    message: e.to_string(),
                    line: 0,
                    column: 0,
                    source_snippet: String::new(),
                }),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_valid_typescript() {
        let compiler = TypeScriptCompiler::new();
        let source = "const x: number = 42; console.log(x);";
        let result = compiler.compile(source);
        assert!(result.is_ok());
        let js = result.unwrap();
        // SWC with ES2020 target should preserve const
        assert!(js.contains("const x = 42"));
        assert!(js.contains("console.log(x)"));
    }

    #[test]
    fn test_compile_invalid_syntax() {
        let compiler = TypeScriptCompiler::new();
        let source = "const x: number = ;"; // Syntax error
        let result = compiler.compile(source);
        assert!(result.is_err());
    }
}
