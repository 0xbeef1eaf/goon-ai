use super::WallpaperSetter;
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct MacOsWallpaperSetter;

impl WallpaperSetter for MacOsWallpaperSetter {
    fn get_wallpaper(&self) -> Result<PathBuf> {
        let script = r#"tell application "System Events" to get picture of first desktop"#;
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| anyhow!("Failed to get macOS wallpaper: {}", e))?;

        let path_str = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(PathBuf::from(path_str))
    }

    fn set_wallpaper(&self, path: &Path) -> Result<()> {
        let path_str = path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;

        let script = format!(
            r#"tell application "System Events" to tell every desktop to set picture to "{}""#,
            path_str
        );

        Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map(|_| ())
            .map_err(|e| anyhow!("Failed to set macOS wallpaper: {}", e))
    }
}
