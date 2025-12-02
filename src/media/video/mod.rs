//! FFmpeg-based video playback module
//!
//! This module provides video playback using FFmpeg (via ffmpeg-next crate)
//! integrated with Slint for display. Video frames are decoded in a background
//! thread and rendered to Slint Image components.

pub mod audio;
pub mod player;

pub use player::{ControlCommand, Player, VideoHandle};
