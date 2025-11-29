use crate::sdk::templates;

pub struct SdkModule {
    pub name: &'static str,
    pub template: String,
    pub permission: Option<&'static str>,
    pub dependencies: Vec<&'static str>,
}

pub fn get_modules() -> Vec<SdkModule> {
    vec![
        SdkModule {
            name: "types",
            template: templates::types_ts(),
            permission: None, // Always included
            dependencies: vec![],
        },
        SdkModule {
            name: "pack",
            template: templates::pack_ts(),
            permission: None, // Always included
            dependencies: vec!["types"],
        },
        SdkModule {
            name: "image",
            template: templates::image_ts(),
            permission: Some("image"),
            dependencies: vec!["types"],
        },
        SdkModule {
            name: "video",
            template: templates::video_ts(),
            permission: Some("video"),
            dependencies: vec!["types"],
        },
        SdkModule {
            name: "audio",
            template: templates::audio_ts(),
            permission: Some("audio"),
            dependencies: vec!["types"],
        },
        SdkModule {
            name: "textPrompt",
            template: templates::prompt_ts(),
            permission: Some("prompt"),
            dependencies: vec!["types", "image"],
        },
        SdkModule {
            name: "wallpaper",
            template: templates::wallpaper_ts(),
            permission: Some("wallpaper"),
            dependencies: vec![],
        },
        SdkModule {
            name: "website",
            template: templates::website_ts(),
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
