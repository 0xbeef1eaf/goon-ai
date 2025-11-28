use crate::sdk::templates;

pub struct SdkModule {
    pub name: &'static str,
    pub template: &'static str,
    pub permission: Option<&'static str>,
    pub dependencies: Vec<&'static str>,
}

pub fn get_modules() -> Vec<SdkModule> {
    vec![
        SdkModule {
            name: "types",
            template: templates::TYPES_TS,
            permission: None, // Always included
            dependencies: vec![],
        },
        SdkModule {
            name: "pack",
            template: templates::PACK_TS,
            permission: None, // Always included
            dependencies: vec!["types"],
        },
        SdkModule {
            name: "image",
            template: templates::IMAGE_TS,
            permission: Some("image"),
            dependencies: vec!["types"],
        },
        SdkModule {
            name: "video",
            template: templates::VIDEO_TS,
            permission: Some("video"),
            dependencies: vec!["types"],
        },
        SdkModule {
            name: "audio",
            template: templates::AUDIO_TS,
            permission: Some("audio"),
            dependencies: vec!["types"],
        },
        SdkModule {
            name: "textPrompt",
            template: templates::PROMPT_TS,
            permission: Some("prompt"),
            dependencies: vec!["types", "image"],
        },
        SdkModule {
            name: "wallpaper",
            template: templates::WALLPAPER_TS,
            permission: Some("wallpaper"),
            dependencies: vec![],
        },
        SdkModule {
            name: "website",
            template: templates::WEBSITE_TS,
            permission: Some("website"),
            dependencies: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_modules_not_empty() {
        let modules = get_modules();
        assert!(!modules.is_empty());
    }

    #[test]
    fn test_modules_have_templates() {
        let modules = get_modules();
        for module in modules {
            assert!(!module.template.is_empty());
            assert!(!module.name.is_empty());
        }
    }

    #[test]
    fn test_specific_module_exists() {
        let modules = get_modules();
        assert!(modules.iter().any(|m| m.name == "image"));
        assert!(modules.iter().any(|m| m.name == "types"));
    }
}
