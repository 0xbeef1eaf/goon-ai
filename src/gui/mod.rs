//! GUI layer for goon.ai
//!
//! Architecture:
//! - System tray (tray.rs) provides Run/Pause, Config, Pack Editor menu items
//! - Window spawner (windows.rs) handles Slint window lifecycle via channels
//! - Each window manages its own lifecycle and can be spawned in large numbers

pub mod tray;
pub mod windows;

pub use tray::{SystemTray, TrayCommand};
pub use windows::{
    WindowCommand, WindowHandle, WindowSpawner, WindowSpawnerHandle, run_event_loop,
};
