use crate::config::pack::PackConfig;
use crate::config::settings::Settings;
use crate::permissions::{PermissionChecker, PermissionResolver, PermissionSet};
use anyhow::Result;

pub struct App {
    pub settings: Settings,
    #[allow(dead_code)]
    pub pack_config: PackConfig,
    pub permissions: PermissionChecker,
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
            println!("Warning: Pack requested permissions that are not granted: {:?}", missing);
        }

        Ok(Self {
            settings,
            pack_config,
            permissions,
        })
    }

    pub fn run(&self) -> Result<()> {
        println!("App running with mood: {}", self.settings.runtime.pack.mood);
        // Main loop will go here (Issue #16)
        Ok(())
    }
}
