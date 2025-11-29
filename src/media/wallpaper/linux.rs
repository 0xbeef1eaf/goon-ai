use super::WallpaperSetter;
use anyhow::{Result, anyhow};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use url::Url;

pub struct LinuxWallpaperSetter;

impl WallpaperSetter for LinuxWallpaperSetter {
    fn get_wallpaper(&self) -> Result<PathBuf> {
        let desktop = env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_lowercase();

        if desktop.contains("gnome") || desktop.contains("unity") || desktop.contains("pantheon") {
            let output = Command::new("gsettings")
                .args(["get", "org.gnome.desktop.background", "picture-uri"])
                .output()
                .map_err(|e| anyhow!("Failed to get wallpaper: {}", e))?;

            let uri_str = String::from_utf8(output.stdout)?
                .trim()
                .trim_matches('\'')
                .to_string();

            if uri_str.is_empty() {
                return Err(anyhow!("Empty wallpaper URI"));
            }

            let path = if uri_str.starts_with("file://") {
                let url = Url::parse(&uri_str)?;
                url.to_file_path()
                    .map_err(|_| anyhow!("Failed to convert URL to file path"))?
            } else {
                PathBuf::from(uri_str)
            };
            Ok(path)
        } else if desktop.contains("kde") || desktop.contains("plasma") {
            let script = r#"
                var allDesktops = desktops();
                for (i=0;i<allDesktops.length;i++) {
                    d = allDesktops[i];
                    d.wallpaperPlugin = "org.kde.image";
                    d.currentConfigGroup = Array("Wallpaper", "org.kde.image", "General");
                    print(d.readConfig("Image"));
                }
            "#;
            let output = Command::new("qdbus")
                .args([
                    "org.kde.plasmashell",
                    "/PlasmaShell",
                    "org.kde.PlasmaShell.evaluateScript",
                    script,
                ])
                .output()
                .map_err(|e| anyhow!("Failed to get KDE wallpaper: {}", e))?;

            let uri_str = String::from_utf8(output.stdout)?.trim().to_string();

            // The script might print multiple lines if there are multiple desktops, just take the first one
            let first_uri = uri_str
                .lines()
                .next()
                .ok_or_else(|| anyhow!("Empty wallpaper URI"))?;

            let path = if first_uri.starts_with("file://") {
                let url = Url::parse(first_uri)?;
                url.to_file_path()
                    .map_err(|_| anyhow!("Failed to convert URL to file path"))?
            } else {
                PathBuf::from(first_uri)
            };
            Ok(path)
        } else {
            // Try to read from nitrogen config
            if let Ok(config_dir) = env::var("XDG_CONFIG_HOME") {
                let config_path = PathBuf::from(config_dir).join("nitrogen/bg-saved.cfg");
                if config_path.exists() {
                    let content = fs::read_to_string(config_path)?;
                    for line in content.lines() {
                        if line.starts_with("file=") {
                            let path_str = line.trim_start_matches("file=");
                            return Ok(PathBuf::from(path_str));
                        }
                    }
                }
            } else if let Ok(home) = env::var("HOME") {
                let config_path = PathBuf::from(home).join(".config/nitrogen/bg-saved.cfg");
                if config_path.exists() {
                    let content = fs::read_to_string(config_path)?;
                    for line in content.lines() {
                        if line.starts_with("file=") {
                            let path_str = line.trim_start_matches("file=");
                            return Ok(PathBuf::from(path_str));
                        }
                    }
                }
            }

            // Try to read from .fehbg
            if let Ok(home) = env::var("HOME") {
                let fehbg_path = PathBuf::from(home).join(".fehbg");
                if fehbg_path.exists() {
                    let content = fs::read_to_string(fehbg_path)?;
                    // Content is like: feh --no-fehbg --bg-scale '/path/to/image'
                    // We need to extract the path, which is usually the last argument and might be quoted
                    for line in content.lines() {
                        if line.starts_with("feh") {
                            // Simple parsing: split by whitespace, take last part, trim quotes
                            if let Some(last_arg) = line.split_whitespace().last() {
                                let path_str = last_arg.trim_matches('\'').trim_matches('"');
                                return Ok(PathBuf::from(path_str));
                            }
                        }
                    }
                }
            }

            Err(anyhow!(
                "Getting wallpaper not supported for this desktop environment: {}",
                desktop
            ))
        }
    }

    fn set_wallpaper(&self, path: &Path) -> Result<()> {
        let path_str = path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
        let desktop = env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_lowercase();

        if desktop.contains("gnome") || desktop.contains("unity") || desktop.contains("pantheon") {
            let uri = format!("file://{}", path_str);
            // Try setting both light and dark mode
            let _ = Command::new("gsettings")
                .args(["set", "org.gnome.desktop.background", "picture-uri", &uri])
                .output();
            let _ = Command::new("gsettings")
                .args([
                    "set",
                    "org.gnome.desktop.background",
                    "picture-uri-dark",
                    &uri,
                ])
                .output();
            Ok(())
        } else if desktop.contains("kde") || desktop.contains("plasma") {
            let script = format!(
                r#"
                var allDesktops = desktops();
                for (i=0;i<allDesktops.length;i++) {{
                    d = allDesktops[i];
                    d.wallpaperPlugin = "org.kde.image";
                    d.currentConfigGroup = Array("Wallpaper", "org.kde.image", "General");
                    d.writeConfig("Image", "file://{}");
                }}
                "#,
                path_str
            );
            Command::new("qdbus")
                .args([
                    "org.kde.plasmashell",
                    "/PlasmaShell",
                    "org.kde.PlasmaShell.evaluateScript",
                    &script,
                ])
                .output()
                .map(|_| ())
                .map_err(|e| anyhow!("Failed to set KDE wallpaper: {}", e))
        } else if desktop.contains("xfce") {
            // Try xfconf-query loop via shell
            let cmd = format!(
                "xfconf-query -c xfce4-desktop -l | grep last-image | while read property; do xfconf-query -c xfce4-desktop -p \"$property\" -s \"{}\"; done",
                path_str
            );
            Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .map(|_| ())
                .map_err(|e| anyhow!("Failed to set XFCE wallpaper: {}", e))
        } else {
            // Fallback to feh or nitrogen
            if Command::new("feh")
                .arg("--bg-scale")
                .arg(path_str)
                .output()
                .is_ok()
            {
                return Ok(());
            }
            if Command::new("nitrogen")
                .args(["--set-scaled", path_str])
                .output()
                .is_ok()
            {
                return Ok(());
            }
            Err(anyhow!(
                "Unsupported desktop environment: {} and no feh/nitrogen found",
                desktop
            ))
        }
    }
}
