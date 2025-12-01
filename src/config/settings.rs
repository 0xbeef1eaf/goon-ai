use crate::permissions::Permission;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub user: User,
    #[serde(rename = "llmSettings")]
    pub llm_settings: LLMSettings,
    pub runtime: RuntimeSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub name: String,
    pub dob: String,
    pub gender: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LLMSettings {
    pub host: String,
    #[serde(default = "default_model")]
    pub model: String,
}

fn default_model() -> String {
    "llama3".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RuntimeSettings {
    pub popups: Popups,
    pub permissions: Vec<Permission>,
    pub pack: PackSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Popups {
    pub image: PopupConfig,
    pub video: PopupConfig,
    pub audio: PopupConfig,
    pub wallpaper: WallpaperConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PopupConfig {
    pub enabled: bool,
    pub timeout: Option<u64>,
    pub max: Option<u32>,
    pub mitosis: Option<MitosisConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MitosisConfig {
    pub enabled: bool,
    pub factor: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WallpaperConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackSettings {
    pub current: String,
    pub mood: String,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let path = Path::new("settings.yaml");
        let content = if path.exists() {
            fs::read_to_string(path).context("Failed to read settings.yaml")?
        } else {
            let example_path = Path::new("settings.example.yaml");
            if example_path.exists() {
                println!("settings.yaml not found, using settings.example.yaml");
                fs::read_to_string(example_path).context("Failed to read settings.example.yaml")?
            } else {
                anyhow::bail!("Neither settings.yaml nor settings.example.yaml found");
            }
        };

        let settings: Settings =
            serde_yaml::from_str(&content).context("Failed to parse settings")?;
        Ok(settings)
    }

    #[allow(dead_code)]
    pub fn parse(content: &str) -> Result<Self> {
        let settings: Settings =
            serde_yaml::from_str(content).context("Failed to parse settings")?;
        Ok(settings)
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_yaml::to_string(self).context("Failed to serialize settings")?;
        fs::write("settings.yaml", content).context("Failed to write settings.yaml")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_settings() {
        let yaml = r#"
user:
  name: Test User
  dob: 1990-01-01
  gender: male
llmSettings:
  host: "http://localhost:11434"
runtime:
  popups:
    image:
      enabled: true
      timeout: 10
      max: 5
      mitosis:
        enabled: false
        factor: 2
    video:
      enabled: true
      timeout: 15
      max: 3
    audio:
      enabled: true
      timeout: 5
      max: 1
    wallpaper:
      enabled: true
  permissions:
    - image
  pack:
    current: Test Pack
    mood: default
"#;
        let settings = Settings::parse(yaml).unwrap();
        assert_eq!(settings.user.name, "Test User");
        assert_eq!(settings.runtime.pack.current, "Test Pack");
        assert_eq!(settings.runtime.permissions, vec![Permission::Image]);
    }
}
