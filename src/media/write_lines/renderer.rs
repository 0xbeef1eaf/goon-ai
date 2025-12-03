//! Prompt rendering module
//!
//! NOTE: This module is being deprecated as part of the transition to a pure Slint-based
//! GUI architecture. Prompt windows are now implemented via the gui::windows module.

/// WriteLines content data (deprecated - use gui::windows::WindowCommand::SpawnWriteLines)
#[allow(dead_code)]
pub struct WriteLinesContent {
    pub text: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub background_color: Option<[f32; 4]>,
    pub max_width: Option<u32>,
    pub alignment: String,
}
