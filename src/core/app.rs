use crate::app_loop::orchestrator::Orchestrator;
use crate::config::pack::PackConfig;
use crate::config::settings::Settings;
use crate::gui::slint_controller::SlintGuiController;
use crate::permissions::{PermissionChecker, PermissionResolver, PermissionSet};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

pub struct App {
    pub settings: Arc<Settings>,
    #[allow(dead_code)]
    pub pack_config: Arc<PackConfig>,
    #[allow(dead_code)]
    pub permissions: Arc<PermissionChecker>,
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
            settings: Arc::new(settings),
            pack_config: Arc::new(pack_config),
            permissions: Arc::new(permissions),
            original_wallpaper,
        })
    }

    pub async fn run(&self) -> Result<()> {
        println!("App running with mood: {}", self.settings.runtime.pack.mood);

        let max_audio = self.settings.runtime.popups.audio.max.unwrap_or(1) as usize;
        let max_video = self.settings.runtime.popups.video.max.unwrap_or(1) as usize;
        println!("Max concurrent audio: {}, video: {}", max_audio, max_video);

        // Initialize Slint GUI controller
        let gui_controller = Arc::new(SlintGuiController::new());

        let mut orchestrator = Orchestrator::new(
            self.settings.clone(),
            self.pack_config.clone(),
            self.permissions.clone(),
            gui_controller.clone(),
        );

        // Schedule the orchestrator to run within the Slint event loop context
        slint::spawn_local(async move {
            if let Err(e) = orchestrator.run().await {
                eprintln!("Orchestrator error: {}", e);
            }
            // Quit the event loop when done
            let _ = slint::quit_event_loop();
        })
        .map_err(|e| anyhow::anyhow!("Failed to spawn orchestrator task: {}", e))?;

        // Run Slint event loop (blocks on main thread)
        SlintGuiController::run_event_loop()?;

        Ok(())
    }

    pub async fn run_script(&self, script: &str) -> Result<()> {
        println!(
            "App running script mode with mood: {}",
            self.settings.runtime.pack.mood
        );

        let max_audio = self.settings.runtime.popups.audio.max.unwrap_or(1) as usize;
        let max_video = self.settings.runtime.popups.video.max.unwrap_or(1) as usize;
        println!("Max concurrent audio: {}, video: {}", max_audio, max_video);

        // Initialize Slint GUI controller
        let gui_controller = Arc::new(SlintGuiController::new());

        let mut orchestrator = Orchestrator::new(
            self.settings.clone(),
            self.pack_config.clone(),
            self.permissions.clone(),
            gui_controller.clone(),
        );

        // Clone script for the closure
        let script = script.to_string();

        // Schedule the orchestrator to run within the Slint event loop context
        // This ensures the Slint platform is initialized before we try to use it
        slint::spawn_local(async move {
            println!("Running script in sandbox...");
            if let Err(e) = orchestrator.run_script(&script).await {
                eprintln!("Orchestrator error: {}", e);
            }
            // Quit the event loop when done
            let _ = slint::quit_event_loop();
        })
        .map_err(|e| anyhow::anyhow!("Failed to spawn script task: {}", e))?;

        // Run Slint event loop (blocks on main thread)
        SlintGuiController::run_event_loop()?;

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
