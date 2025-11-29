#![allow(dead_code)]

use crate::permissions::{Permission, PermissionChecker};

pub mod audio;
pub mod hypno;
pub mod image;
pub mod prompt;
pub mod system;
pub mod video;
pub mod wallpaper;
pub mod website;

pub mod analysis;
pub mod generator;
pub mod metadata;
pub mod templates;
pub mod types;

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

pub fn generate_definitions_for_permissions(permissions: &PermissionChecker) -> String {
    let mut allowed_modules = Vec::new();
    if permissions.has_permission(Permission::Image) {
        allowed_modules.push("image".to_string());
    }
    if permissions.has_permission(Permission::Video) {
        allowed_modules.push("video".to_string());
    }
    if permissions.has_permission(Permission::Audio) {
        allowed_modules.push("audio".to_string());
    }
    if permissions.has_permission(Permission::Hypno) {
        allowed_modules.push("hypno".to_string());
    }
    if permissions.has_permission(Permission::Wallpaper) {
        allowed_modules.push("wallpaper".to_string());
    }
    if permissions.has_permission(Permission::Prompt) {
        allowed_modules.push("prompt".to_string());
    }
    if permissions.has_permission(Permission::Website) {
        allowed_modules.push("website".to_string());
    }

    generator::generate_definitions(&allowed_modules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{Permission, PermissionChecker, PermissionSet};

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

    #[test]
    fn test_generate_definitions_for_permissions() {
        let mut set = PermissionSet::new();
        set.add(Permission::Image);
        let checker = PermissionChecker::new(set);

        let defs = generate_definitions_for_permissions(&checker);
        assert!(defs.contains("class image"));
        assert!(!defs.contains("class video"));
    }
}
