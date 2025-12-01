use crate::permissions::Permission;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackConfig {
    pub meta: PackMeta,
    pub moods: Vec<Mood>,
    pub assets: Assets,
    pub websites: Option<Vec<WebsiteConfig>>,
    pub prompts: Option<PromptsConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromptsConfig {
    pub system: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebsiteConfig {
    pub name: String,
    pub url: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackMeta {
    pub name: String,
    pub version: String,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Mood {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub prompt: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Assets {
    pub image: Option<Vec<Asset>>,
    pub video: Option<Vec<Asset>>,
    pub audio: Option<Vec<Asset>>,
    pub hypno: Option<Vec<Asset>>,
    pub wallpaper: Option<Vec<Asset>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Asset {
    pub path: String,
    pub tags: Vec<String>,
}

impl PackConfig {
    pub fn load(pack_name: &str) -> Result<Self> {
        let path = Path::new("packs").join(pack_name).join("config.yaml");
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read pack config at {:?}", path))?;

        let config: PackConfig = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse pack config for {}", pack_name))?;
        Ok(config)
    }

    #[allow(dead_code)]
    pub fn parse(content: &str) -> Result<Self> {
        let config: PackConfig =
            serde_yaml::from_str(content).context("Failed to parse pack config")?;
        Ok(config)
    }

    pub fn save(&self, pack_name: &str) -> Result<()> {
        let path = Path::new("packs").join(pack_name).join("config.yaml");
        let content = serde_yaml::to_string(self).context("Failed to serialize pack config")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write pack config to {:?}", path))?;
        Ok(())
    }

    pub fn new(name: &str) -> Self {
        Self {
            meta: PackMeta {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                permissions: vec![],
            },
            moods: vec![Mood {
                name: "default".to_string(),
                description: "Default mood".to_string(),
                tags: vec![],
                prompt: None,
            }],
            assets: Assets {
                image: Some(vec![]),
                video: Some(vec![]),
                audio: Some(vec![]),
                hypno: Some(vec![]),
                wallpaper: Some(vec![]),
            },
            websites: Some(vec![]),
            prompts: Some(PromptsConfig {
                system: Some(
                    "You are an AI assistant designed to help test the functionality of goon.ai."
                        .to_string(),
                ),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pack_config() {
        let yaml = r#"
meta:
  name: Test Pack
  version: 1.0.0
  permissions:
    - image
moods:
  - name: default
    description: Default mood
    tags:
      - test
assets:
  image:
    - path: image/test.jpg
      tags:
        - test
"#;
        let config = PackConfig::parse(yaml).unwrap();
        assert_eq!(config.meta.name, "Test Pack");
        assert_eq!(config.moods[0].name, "default");
        assert_eq!(
            config.assets.image.as_ref().unwrap()[0].path,
            "image/test.jpg"
        );
    }
}
