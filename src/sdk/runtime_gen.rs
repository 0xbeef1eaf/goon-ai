/// TypeScript runtime code generator
///
/// This module generates TypeScript runtime classes (Handle classes and SDK classes)
/// from Rust op definitions. The generated code includes:
/// - Handle classes with id property and methods (e.g., close())
/// - SDK classes with static async methods that call Deno.core.ops
/// - GlobalThis registration for the goon namespace
use crate::sdk::analysis::{self, OpInfo};
use std::path::Path;

/// Configuration for generating a module's runtime code
#[derive(Default)]
pub struct ModuleConfig {
    /// The module name (e.g., "image", "video", "audio")
    pub name: &'static str,
    /// The class name for the SDK (e.g., "image", "video", "audio")
    pub class_name: &'static str,
    /// Whether this module uses handles (returns an ID that can be used for close/control)
    pub has_handle: bool,
    /// The handle class name if has_handle is true (e.g., "ImageHandle", "VideoHandle")
    pub handle_class_name: Option<&'static str>,
    /// Custom handle class code (overrides default generation)
    pub custom_handle_code: Option<&'static str>,
    /// Custom primary method body (for special return handling)
    pub custom_primary_body: Option<&'static str>,
    /// The primary op name (e.g., "op_show_image", "op_play_audio")
    pub primary_op: &'static str,
    /// The primary method name in TypeScript (e.g., "show", "play")
    pub primary_method: &'static str,
    /// Whether the primary method returns a value (not void/handle)
    pub primary_returns_value: bool,
    /// The options type name (e.g., "ImageOptions", "VideoOptions") - None for no options
    pub options_type: Option<&'static str>,
    /// Additional methods to generate
    pub extra_methods: Vec<MethodConfig>,
    /// The source file path for documentation extraction
    pub source_path: &'static str,
}

/// Configuration for a single method
#[derive(Clone, Default)]
pub struct MethodConfig {
    /// The Rust op name (e.g., "op_get_asset")
    pub op_name: &'static str,
    /// The TypeScript method name (e.g., "getAsset")
    pub method_name: &'static str,
    /// Parameter name if the method takes an argument
    pub param_name: Option<&'static str>,
    /// Whether the method is synchronous
    pub is_sync: bool,
    /// Whether the method returns a value (not void)
    pub returns_value: bool,
}

/// Convert a Rust op function name to a TypeScript method name.
/// e.g., "op_show_image" -> "showImage"
fn op_name_to_ts_method(op_name: &str) -> String {
    let name = op_name.strip_prefix("op_").unwrap_or(op_name);
    let parts: Vec<&str> = name.split('_').collect();
    if parts.is_empty() {
        return name.to_string();
    }

    let mut result = parts[0].to_string();
    for part in &parts[1..] {
        if !part.is_empty() {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                result.push_str(&first.to_uppercase().to_string());
                result.push_str(chars.as_str());
            }
        }
    }
    result
}

/// Generate a Handle class that wraps a window/media handle ID
fn generate_handle_class(handle_name: &str, close_op: &str) -> String {
    format!(
        r#"class {handle_name} {{
    constructor(id) {{
        this.id = id;
    }}

    /**
     * Closes this handle and releases associated resources.
     */
    async close() {{
        await Deno.core.ops.{close_op}(this.id);
    }}
}}
"#,
        handle_name = handle_name,
        close_op = close_op
    )
}

/// Generate JSDoc comment from op documentation
fn generate_jsdoc(docs: &[String], indent: &str) -> String {
    if docs.is_empty() {
        return String::new();
    }

    let mut jsdoc = format!("{}/**\n", indent);
    for doc in docs {
        jsdoc.push_str(&format!("{} * {}\n", indent, doc));
    }
    jsdoc.push_str(&format!("{} */\n", indent));
    jsdoc
}

/// Find documentation for an op from analyzed source
fn find_op_docs(ops: &[OpInfo], op_name: &str) -> Vec<String> {
    ops.iter()
        .find(|op| op.name == op_name)
        .map(|op| op.docs.clone())
        .unwrap_or_default()
}

/// Generate a static async method that calls a Deno op
fn generate_method(
    method_name: &str,
    op_name: &str,
    param_name: Option<&str>,
    return_handle: Option<&str>,
    docs: &[String],
    is_sync: bool,
) -> String {
    let jsdoc = generate_jsdoc(docs, "    ");

    let params = param_name.unwrap_or("").to_string();
    let args = params.clone();

    let body = match return_handle {
        Some(handle) => format!(
            r#"const id = await Deno.core.ops.{}({});
        return new {}(id);"#,
            op_name, args, handle
        ),
        None if is_sync => format!("Deno.core.ops.{}({});", op_name, args),
        None => format!("await Deno.core.ops.{}({});", op_name, args),
    };

    let async_keyword = if is_sync { "" } else { "async " };

    format!(
        r#"{}    static {}{}({}) {{
        {}
    }}
"#,
        jsdoc, async_keyword, method_name, params, body
    )
}

/// Generate a method that returns a value
fn generate_returning_method(
    method_name: &str,
    op_name: &str,
    param_name: Option<&str>,
    docs: &[String],
    is_sync: bool,
) -> String {
    let jsdoc = generate_jsdoc(docs, "    ");

    let params = param_name.unwrap_or("").to_string();
    let args = params.clone();

    let (body, async_keyword) = if is_sync {
        (format!("return Deno.core.ops.{}({});", op_name, args), "")
    } else {
        (
            format!("return await Deno.core.ops.{}({});", op_name, args),
            "async ",
        )
    };

    format!(
        r#"{}    static {}{}({}) {{
        {}
    }}
"#,
        jsdoc, async_keyword, method_name, params, body
    )
}

/// Generate a void method (no handle return)
fn generate_void_method(
    method_name: &str,
    op_name: &str,
    param_name: Option<&str>,
    docs: &[String],
) -> String {
    generate_method(method_name, op_name, param_name, None, docs, false)
}

/// Generate a sync void method
fn generate_sync_void_method(
    method_name: &str,
    op_name: &str,
    param_name: Option<&str>,
    docs: &[String],
) -> String {
    generate_method(method_name, op_name, param_name, None, docs, true)
}

/// Generate the globalThis registration code
fn generate_global_registration(class_name: &str) -> String {
    format!(
        r#"
(globalThis as any).goon = (globalThis as any).goon || {{}};
(globalThis as any).goon.{class_name} = {class_name};
"#,
        class_name = class_name
    )
}

/// Generate the complete runtime code for a module
pub fn generate_module_runtime(config: &ModuleConfig) -> String {
    let mut output = String::new();
    output.push_str("// @ts-nocheck\n\n");

    // Analyze source for documentation
    let ops = if Path::new(config.source_path).exists() {
        let (ops, _) = analysis::analyze_source(Path::new(config.source_path));
        ops
    } else {
        Vec::new()
    };

    // Generate handle class if needed
    if config.has_handle {
        if let Some(custom_code) = config.custom_handle_code {
            output.push_str(custom_code);
            output.push('\n');
        } else if let Some(handle_name) = config.handle_class_name {
            output.push_str(&generate_handle_class(handle_name, "op_close_window"));
            output.push('\n');
        }
    }

    // Generate main class
    output.push_str(&format!("class {} {{\n", config.class_name));

    // Generate primary method if specified
    if !config.primary_op.is_empty() {
        let primary_docs = find_op_docs(&ops, config.primary_op);
        if let Some(custom_body) = config.custom_primary_body {
            // Use custom body for primary method
            let jsdoc = generate_jsdoc(&primary_docs, "    ");
            let params = config.options_type.map(|_| "options").unwrap_or("");
            output.push_str(&format!(
                "{}    static async {}({}) {{\n        {}\n    }}\n",
                jsdoc, config.primary_method, params, custom_body
            ));
        } else {
            let param = config.options_type.map(|_| "options");
            let primary_method = if config.primary_returns_value {
                // Primary method returns a value (not void, not handle)
                generate_returning_method(
                    config.primary_method,
                    config.primary_op,
                    param,
                    &primary_docs,
                    false,
                )
            } else {
                // Primary method may return a handle or void
                generate_method(
                    config.primary_method,
                    config.primary_op,
                    param,
                    config.handle_class_name,
                    &primary_docs,
                    false,
                )
            };
            output.push_str(&primary_method);
        }
    }

    // Generate extra methods
    for method in &config.extra_methods {
        let docs = find_op_docs(&ops, method.op_name);
        let generated = if method.returns_value {
            generate_returning_method(
                method.method_name,
                method.op_name,
                method.param_name,
                &docs,
                method.is_sync,
            )
        } else if method.is_sync {
            generate_sync_void_method(method.method_name, method.op_name, method.param_name, &docs)
        } else {
            generate_void_method(method.method_name, method.op_name, method.param_name, &docs)
        };
        output.push_str(&generated);
    }

    output.push_str("}\n");

    // Generate global registration
    output.push_str(&generate_global_registration(config.class_name));

    output
}

/// Generate the image module runtime
pub fn generate_image_runtime() -> String {
    generate_module_runtime(&ModuleConfig {
        name: "image",
        class_name: "image",
        has_handle: true,
        handle_class_name: Some("ImageHandle"),
        custom_handle_code: None,
        custom_primary_body: None,
        primary_op: "op_show_image",
        primary_method: "show",
        primary_returns_value: false,
        options_type: Some("ImageOptions"),
        extra_methods: vec![],
        source_path: "src/sdk/image.rs",
    })
}

/// Generate the video module runtime
pub fn generate_video_runtime() -> String {
    generate_module_runtime(&ModuleConfig {
        name: "video",
        class_name: "video",
        has_handle: true,
        handle_class_name: Some("VideoHandle"),
        custom_handle_code: None,
        custom_primary_body: None,
        primary_op: "op_show_video",
        primary_method: "play",
        primary_returns_value: false,
        options_type: Some("VideoOptions"),
        extra_methods: vec![],
        source_path: "src/sdk/video.rs",
    })
}

/// Generate the audio module runtime
pub fn generate_audio_runtime() -> String {
    generate_module_runtime(&ModuleConfig {
        name: "audio",
        class_name: "audio",
        has_handle: true,
        handle_class_name: Some("AudioHandle"),
        custom_handle_code: Some(
            r#"class AudioHandle {
    constructor(handle) {
        this.id = handle.id;
    }

    async stop() {
        await Deno.core.ops.op_stop_audio(this.id);
    }

    async pause() {
        await Deno.core.ops.op_pause_audio(this.id);
    }

    async resume() {
        await Deno.core.ops.op_resume_audio(this.id);
    }
}"#,
        ),
        custom_primary_body: Some(
            "const handle = await Deno.core.ops.op_play_audio(options);\n        return new AudioHandle(handle);",
        ),
        primary_op: "op_play_audio",
        primary_method: "play",
        primary_returns_value: false,
        options_type: Some("AudioOptions"),
        extra_methods: vec![],
        source_path: "src/sdk/audio.rs",
    })
}

/// Generate the system module runtime
pub fn generate_system_runtime() -> String {
    generate_module_runtime(&ModuleConfig {
        name: "system",
        class_name: "system",
        has_handle: false,
        handle_class_name: None,
        custom_handle_code: None,
        custom_primary_body: None,
        primary_op: "",
        primary_method: "",
        primary_returns_value: false,
        options_type: None,
        extra_methods: vec![
            MethodConfig {
                op_name: "op_get_asset",
                method_name: "getAsset",
                param_name: Some("tag"),
                is_sync: false,
                returns_value: true,
            },
            MethodConfig {
                op_name: "op_close_window",
                method_name: "closeWindow",
                param_name: Some("handleId"),
                is_sync: false,
                returns_value: false,
            },
            MethodConfig {
                op_name: "op_log",
                method_name: "log",
                param_name: Some("message"),
                is_sync: true,
                returns_value: false,
            },
        ],
        source_path: "src/sdk/system.rs",
    })
}

/// Generate the pack module runtime
pub fn generate_pack_runtime() -> String {
    generate_module_runtime(&ModuleConfig {
        name: "pack",
        class_name: "pack",
        has_handle: false,
        handle_class_name: None,
        custom_handle_code: None,
        custom_primary_body: None,
        primary_op: "op_get_current_mood",
        primary_method: "getCurrentMood",
        primary_returns_value: true,
        options_type: None,
        extra_methods: vec![MethodConfig {
            op_name: "op_set_current_mood",
            method_name: "setMood",
            param_name: Some("moodName"),
            is_sync: false,
            returns_value: false,
        }],
        source_path: "src/sdk/pack.rs",
    })
}

/// Generate the prompt module runtime
pub fn generate_prompt_runtime() -> String {
    generate_module_runtime(&ModuleConfig {
        name: "prompt",
        class_name: "textPrompt",
        has_handle: true,
        handle_class_name: Some("PromptHandle"),
        custom_handle_code: None,
        custom_primary_body: None,
        primary_op: "op_show_prompt",
        primary_method: "show",
        primary_returns_value: false,
        options_type: Some("PromptOptions"),
        extra_methods: vec![],
        source_path: "src/sdk/prompt.rs",
    })
}

/// Generate the wallpaper module runtime
pub fn generate_wallpaper_runtime() -> String {
    generate_module_runtime(&ModuleConfig {
        name: "wallpaper",
        class_name: "wallpaper",
        has_handle: false,
        handle_class_name: None,
        custom_handle_code: None,
        custom_primary_body: None,
        primary_op: "op_set_wallpaper",
        primary_method: "set",
        primary_returns_value: false,
        options_type: Some("WallpaperOptions"),
        extra_methods: vec![],
        source_path: "src/sdk/wallpaper.rs",
    })
}

/// Generate the website module runtime
pub fn generate_website_runtime() -> String {
    generate_module_runtime(&ModuleConfig {
        name: "website",
        class_name: "website",
        has_handle: false,
        handle_class_name: None,
        custom_handle_code: None,
        custom_primary_body: None,
        primary_op: "op_open_website",
        primary_method: "open",
        primary_returns_value: false,
        options_type: Some("WebsiteOptions"),
        extra_methods: vec![],
        source_path: "src/sdk/website.rs",
    })
}

/// Generate the hypno module runtime
pub fn generate_hypno_runtime() -> String {
    generate_module_runtime(&ModuleConfig {
        name: "hypno",
        class_name: "hypno",
        has_handle: false,
        handle_class_name: None,
        custom_handle_code: None,
        custom_primary_body: None,
        primary_op: "op_show_hypno",
        primary_method: "show",
        primary_returns_value: false,
        options_type: Some("HypnoOptions"),
        extra_methods: vec![],
        source_path: "src/sdk/hypno.rs",
    })
}

/// Generate the init module runtime
pub fn generate_init_runtime() -> String {
    r#"// Initialize the global goon namespace
if (!(globalThis as any).goon) {
    (globalThis as any).goon = {};
}
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_name_to_ts_method() {
        assert_eq!(op_name_to_ts_method("op_show_image"), "showImage");
        assert_eq!(op_name_to_ts_method("op_play_audio"), "playAudio");
        assert_eq!(op_name_to_ts_method("op_set_mood"), "setMood");
        assert_eq!(op_name_to_ts_method("op_show"), "show");
    }

    #[test]
    fn test_generate_handle_class() {
        let output = generate_handle_class("ImageHandle", "op_close_window");
        assert!(output.contains("class ImageHandle"));
        assert!(output.contains("this.id = id"));
        assert!(output.contains("async close()"));
        assert!(output.contains("op_close_window"));
    }

    #[test]
    fn test_generate_image_runtime() {
        let output = generate_image_runtime();
        assert!(output.contains("class ImageHandle"));
        assert!(output.contains("class image"));
        assert!(output.contains("static async show"));
        assert!(output.contains("op_show_image"));
        assert!(output.contains("goon.image = image"));
    }

    #[test]
    fn test_generate_audio_runtime() {
        let output = generate_audio_runtime();
        assert!(output.contains("class AudioHandle"));
        assert!(output.contains("class audio"));
        assert!(output.contains("static async play"));
        assert!(output.contains("op_play_audio"));
    }

    #[test]
    fn test_generate_system_runtime() {
        let output = generate_system_runtime();
        assert!(output.contains("class system"));
        assert!(output.contains("static async getAsset"));
        assert!(output.contains("static log"));
    }

    #[test]
    fn test_no_import_statements_in_all_generated_sources() {
        use crate::sdk;
        let sources = sdk::get_all_typescript_sources();
        for (i, source) in sources.iter().enumerate() {
            if source.contains("import ") {
                panic!(
                    "Source {} contains import statement:\n{}",
                    i,
                    source.lines().take(10).collect::<Vec<_>>().join("\n")
                );
            }
        }
    }

    #[test]
    fn test_compiled_output_no_imports() {
        use crate::sdk;
        use crate::typescript::TypeScriptCompiler;

        let compiler = TypeScriptCompiler::new();
        let sources = sdk::get_all_typescript_sources();

        for (i, source) in sources.iter().enumerate() {
            let result = compiler.compile(source);
            match result {
                Ok(js_code) => {
                    if js_code.contains("import ") {
                        panic!(
                            "Compiled source {} contains import statement:\n{}",
                            i,
                            js_code.lines().take(10).collect::<Vec<_>>().join("\n")
                        );
                    }
                }
                Err(e) => {
                    panic!("Failed to compile source {}: {:?}", i, e);
                }
            }
        }
    }
}
