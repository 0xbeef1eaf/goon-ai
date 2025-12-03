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
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PopupConfig {
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
pub struct PackSettings {
    pub current: String,
    pub mood: String,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let path = Path::new("settings.toml");
        let content = if path.exists() {
            fs::read_to_string(path).context("Failed to read settings.toml")?
        } else {
            let example_path = Path::new("settings.example.toml");
            if example_path.exists() {
                println!("settings.toml not found, using settings.example.toml");
                fs::read_to_string(example_path).context("Failed to read settings.example.toml")?
            } else {
                anyhow::bail!("Neither settings.toml nor settings.example.toml found");
            }
        };

        let settings: Settings =
            toml::from_str(&content).context(format!("Failed to parse settings: {}", content))?;
        Ok(settings)
    }

    #[allow(dead_code)]
    pub fn parse(content: &str) -> Result<Self> {
        let settings: Settings =
            toml::from_str(content).context(format!("Failed to parse settings: {}", content))?;
        Ok(settings)
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string(self).context("Failed to serialize settings")?;
        fs::write("settings.toml", content).context("Failed to write settings.toml")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_settings() {
        let toml = r#"
[user]
name = "Test User"
dob = "1990-01-01"
gender = "male"

[llmSettings]
host = "http://localhost:11434"

[runtime.popups.image]
max = 5

[runtime.popups.image.mitosis]
enabled = false
factor = 2

[runtime.popups.video]
max = 3

[runtime.popups.audio]
timeout = 5
max = 1

[runtime]
permissions = ["image"]

[runtime.pack]
current = "Test Pack"
mood = "default"
"#;
        let settings = Settings::parse(toml).unwrap();
        assert_eq!(settings.user.name, "Test User");
        assert_eq!(settings.runtime.pack.current, "Test Pack");
        assert_eq!(settings.runtime.permissions, vec![Permission::Image]);
    }
}
