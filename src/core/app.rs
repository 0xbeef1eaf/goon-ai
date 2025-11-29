use crate::config::pack::PackConfig;
use crate::config::settings::Settings;
use crate::permissions::{PermissionChecker, PermissionResolver, PermissionSet};
use anyhow::Result;
use std::path::PathBuf;

pub struct App {
    pub settings: Settings,
    #[allow(dead_code)]
    pub pack_config: PackConfig,
    #[allow(dead_code)]
    pub permissions: PermissionChecker,
    original_wallpaper: Option<PathBuf>,
}

impl App {
    pub fn new() -> Result<Self> {
        // 1. Load Settings
        let settings = Settings::load()?;
        println!("Loaded settings for user: {}", settings.user.name);

        // 2. Load Pack Config
        let pack_name = &settings.runtime.pack.current;
        let pack_config = PackConfig::load(pack_name)?;
        println!(
            "Loaded pack: {} v{}",
            pack_config.meta.name, pack_config.meta.version
        );

        // 3. Compute Permissions
        let user_perms: PermissionSet = settings.runtime.permissions.clone().into();
        let pack_perms: PermissionSet = pack_config.meta.permissions.clone().into();

        let active_perms = PermissionResolver::resolve(&pack_perms, &user_perms);
        let permissions = PermissionChecker::new(active_perms);

        // Check for missing permissions (optional logging)
        let missing = PermissionResolver::find_missing(&pack_perms, &user_perms);
        if !missing.is_empty() {
            println!(
                "Warning: Pack requested permissions that are not granted: {:?}",
                missing
            );
        }

        // 4. Backup Wallpaper (if permission granted)
        let original_wallpaper =
            if permissions.has_permission(crate::permissions::Permission::Wallpaper) {
                let setter = crate::media::wallpaper::PlatformWallpaperSetter;
                use crate::media::wallpaper::WallpaperSetter;
                match setter.get_wallpaper() {
                    Ok(path) => {
                        println!("Backed up wallpaper: {:?}", path);
                        Some(path)
                    }
                    Err(e) => {
                        eprintln!("Failed to backup wallpaper: {}", e);
                        None
                    }
                }
            } else {
                None
            };

        Ok(Self {
            settings,
            pack_config,
            permissions,
            original_wallpaper,
        })
    }

    pub fn run(&self) -> Result<()> {
        println!("App running with mood: {}", self.settings.runtime.pack.mood);

        let max_audio = self.settings.runtime.popups.audio.max.unwrap_or(1) as usize;
        let max_video = self.settings.runtime.popups.video.max.unwrap_or(1) as usize;
        println!("Max concurrent audio: {}, video: {}", max_audio, max_video);

        // Main loop will go here (Issue #16)
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(path) = &self.original_wallpaper {
            println!("Restoring wallpaper: {:?}", path);
            let setter = crate::media::wallpaper::PlatformWallpaperSetter;
            use crate::media::wallpaper::WallpaperSetter;
            if let Err(e) = setter.set_wallpaper(path) {
                eprintln!("Failed to restore wallpaper: {}", e);
            }
        }
    }
}
