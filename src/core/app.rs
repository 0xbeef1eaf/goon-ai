use crate::config::pack::PackConfig;
use crate::config::settings::Settings;
use anyhow::Result;

pub struct App {
    pub settings: Settings,
    #[allow(dead_code)]
    pub pack_config: PackConfig,
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

        // 3. TODO: Compute Permissions (Issue #8)

        Ok(Self {
            settings,
            pack_config,
        })
    }

    pub fn run(&self) -> Result<()> {
        println!("App running with mood: {}", self.settings.runtime.pack.mood);
        // Main loop will go here (Issue #16)
        Ok(())
    }
}
