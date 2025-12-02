//! System tray icon and menu for goon.ai
//!
//! Provides:
//! - Run/Pause toggle for LLM loop
//! - Configuration window launcher
//! - Pack editor window launcher

use anyhow::Result;
use std::sync::mpsc::{Receiver, Sender, channel};
use tracing::info;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};

/// Commands that can be triggered from the system tray
#[derive(Debug, Clone)]
pub enum TrayCommand {
    /// Toggle the LLM loop between running and paused
    ToggleRunPause,
    /// Open the configuration window
    OpenConfig,
    /// Open the pack editor window
    OpenPackEditor,
    /// Quit the application
    Quit,
}

/// System tray manager
pub struct SystemTray {
    _tray_icon: TrayIcon,
    command_rx: Receiver<TrayCommand>,
    run_pause_item: MenuItem,
    is_running: bool,
}

impl SystemTray {
    /// Create a new system tray icon with menu
    pub fn new() -> Result<Self> {
        let (command_tx, command_rx) = channel();

        // Create menu items
        let run_pause_item = MenuItem::new("▶ Run", true, None);
        let config_item = MenuItem::new("⚙ Configuration", true, None);
        let pack_editor_item = MenuItem::new("📦 Pack Editor", true, None);
        let quit_item = MenuItem::new("✕ Quit", true, None);

        // Build menu
        let menu = Menu::new();
        menu.append(&run_pause_item)?;
        menu.append(&config_item)?;
        menu.append(&pack_editor_item)?;
        menu.append(&quit_item)?;

        // Load icon (placeholder - we'll use a simple colored icon)
        let icon = Self::create_default_icon()?;

        // Build tray icon
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("goon.ai")
            .with_icon(icon)
            .build()?;

        // Set up menu event handler
        let run_pause_id = run_pause_item.id().clone();
        let config_id = config_item.id().clone();
        let pack_editor_id = pack_editor_item.id().clone();
        let quit_id = quit_item.id().clone();

        std::thread::spawn(move || {
            Self::menu_event_loop(command_tx, run_pause_id, config_id, pack_editor_id, quit_id);
        });

        info!("System tray initialized");

        Ok(Self {
            _tray_icon: tray_icon,
            command_rx,
            run_pause_item,
            is_running: false,
        })
    }

    /// Create a simple default icon
    fn create_default_icon() -> Result<tray_icon::Icon> {
        // Create a simple 32x32 RGBA icon (purple square)
        let size = 32u32;
        let mut rgba = Vec::with_capacity((size * size * 4) as usize);
        for _ in 0..(size * size) {
            rgba.push(128); // R
            rgba.push(0); // G
            rgba.push(255); // B
            rgba.push(255); // A
        }
        tray_icon::Icon::from_rgba(rgba, size, size)
            .map_err(|e| anyhow::anyhow!("Failed to create icon: {}", e))
    }

    /// Menu event loop running in background thread
    fn menu_event_loop(
        tx: Sender<TrayCommand>,
        run_pause_id: muda::MenuId,
        config_id: muda::MenuId,
        pack_editor_id: muda::MenuId,
        quit_id: muda::MenuId,
    ) {
        loop {
            if let Ok(event) = MenuEvent::receiver().recv() {
                let cmd = if event.id == run_pause_id {
                    TrayCommand::ToggleRunPause
                } else if event.id == config_id {
                    TrayCommand::OpenConfig
                } else if event.id == pack_editor_id {
                    TrayCommand::OpenPackEditor
                } else if event.id == quit_id {
                    TrayCommand::Quit
                } else {
                    continue;
                };

                if tx.send(cmd).is_err() {
                    break;
                }
            }
        }
    }

    /// Poll for tray commands (non-blocking)
    pub fn poll_command(&self) -> Option<TrayCommand> {
        self.command_rx.try_recv().ok()
    }

    /// Update the run/pause menu item text
    pub fn set_running(&mut self, running: bool) {
        self.is_running = running;
        let text = if running { "⏸ Pause" } else { "▶ Run" };
        self.run_pause_item.set_text(text);
    }

    /// Check if currently running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}
