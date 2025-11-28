#![allow(dead_code)]

pub mod audio;
pub mod hypno;
pub mod image;
pub mod prompt;
pub mod system;
pub mod video;
pub mod wallpaper;
pub mod website;

pub mod generator;
pub mod metadata;
pub mod templates;

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
    generator::generate_definitions(allowed_modules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_definitions_empty() {
        let modules = vec![];
        let defs = generate_typescript_definitions(&modules);
        assert!(defs.contains("/** GoonAI SDK */"));
        assert!(defs.contains("interface WindowHandle")); // From types.ts (always included)
        assert!(defs.contains("class pack")); // From pack.ts (always included)
        assert!(!defs.contains("class image"));
    }

    #[test]
    fn test_generate_definitions_image() {
        let modules = vec!["image".to_string()];
        let defs = generate_typescript_definitions(&modules);
        assert!(defs.contains("class image"));
        assert!(defs.contains("static async show"));
    }

    #[test]
    fn test_generate_definitions_video() {
        let modules = vec!["video".to_string()];
        let defs = generate_typescript_definitions(&modules);
        assert!(defs.contains("class video"));
        assert!(defs.contains("static async play"));
        assert!(!defs.contains("class image"));
    }

    #[test]
    fn test_generate_definitions_all() {
        let modules = vec!["all".to_string()];
        let defs = generate_typescript_definitions(&modules);
        assert!(defs.contains("class image"));
        assert!(defs.contains("class video"));
        assert!(defs.contains("class audio"));
        assert!(defs.contains("class textPrompt"));
        assert!(defs.contains("class wallpaper"));
        assert!(defs.contains("class website"));
    }

    #[test]
    fn test_generate_definitions_multiple() {
        let modules = vec!["image".to_string(), "audio".to_string()];
        let defs = generate_typescript_definitions(&modules);
        assert!(defs.contains("class image"));
        assert!(defs.contains("class audio"));
        assert!(!defs.contains("class video"));
    }
}
