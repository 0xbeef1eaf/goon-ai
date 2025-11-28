#![allow(dead_code)]

pub mod image;
pub mod video;
pub mod audio;
pub mod hypno;
pub mod wallpaper;
pub mod prompt;
pub mod website;
pub mod system;

pub const INIT_SOURCE: &str = include_str!("js/init.ts");

pub fn get_all_typescript_sources() -> Vec<&'static str> {
    vec![
        INIT_SOURCE,
        image::TS_SOURCE,
        video::TS_SOURCE,
        audio::TS_SOURCE,
        hypno::TS_SOURCE,
        wallpaper::TS_SOURCE,
        prompt::TS_SOURCE,
        website::TS_SOURCE,
        system::TS_SOURCE,
    ]
}

pub fn generate_typescript_definitions(allowed_modules: &[String]) -> String {
    let mut definitions = String::new();
    
    definitions.push_str("/** GoonAI SDK */\n");
    definitions.push_str("type WindowHandle = number;\n");
    definitions.push_str("declare namespace goon {\n");
    
    if allowed_modules.contains(&"image".to_string()) {
         definitions.push_str(r#"
    namespace image {
        /**
         * Display an image on screen
         * @param path - Asset path or tag query
         * @param options - Display options
         */
        function show(
            path: string,
            options?: {
              duration?: number;
              opacity?: number;
              position?: { x: number; y: number };
              alwaysOnTop?: boolean;
            }
        ): Promise<WindowHandle>;
    }
"#);
    }

    // Add other modules...
    
    definitions.push_str("}\n");
    
    definitions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_definitions_empty() {
        let modules = vec![];
        let defs = generate_typescript_definitions(&modules);
        assert!(defs.contains("/** GoonAI SDK */"));
        assert!(defs.contains("type WindowHandle = number;"));
        assert!(defs.contains("declare namespace goon {"));
        assert!(!defs.contains("namespace image"));
    }

    #[test]
    fn test_generate_definitions_image() {
        let modules = vec!["image".to_string()];
        let defs = generate_typescript_definitions(&modules);
        assert!(defs.contains("namespace image"));
        assert!(defs.contains("function show"));
    }
}
