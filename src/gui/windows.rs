//! Window spawning and lifecycle management for goon.ai
//!
//! Uses channels to communicate between the LLM/eval thread and the Slint UI thread.
//! Each window handles its own lifecycle and cleanup.

pub mod image;
pub mod spawner;
pub mod types;
pub mod video;
pub mod write_lines;

// Import the generated Slint modules
slint::include_modules!();

pub use spawner::{WindowSpawner, WindowSpawnerHandle, run_event_loop};
pub use types::{WindowCommand, WindowHandle, WindowResponse};
