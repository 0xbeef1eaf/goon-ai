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
