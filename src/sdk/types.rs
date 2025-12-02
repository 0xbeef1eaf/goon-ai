use serde::Deserialize;
use ts_rs::TS;

/// Screen position coordinates
#[derive(Deserialize, Debug, Clone, TS)]
pub struct Position {
    /// X coordinate in pixels from the left edge of the screen
    pub x: i32,
    /// Y coordinate in pixels from the top edge of the screen
    pub y: i32,
}

/// Window or element dimensions
#[derive(Deserialize, Debug, Clone, TS)]
pub struct Size {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

/// Common window configuration options
#[derive(Deserialize, Debug, Default, Clone, TS)]
#[serde(rename_all = "camelCase")]
pub struct WindowOptions {
    /// Window opacity from 0.0 (transparent) to 1.0 (opaque)
    pub opacity: Option<f32>,
    /// Initial window position on screen
    pub position: Option<Position>,
    /// Initial window size
    pub size: Option<Size>,
    /// Whether the window should stay above other windows
    pub always_on_top: Option<bool>,
    /// Whether mouse clicks pass through the window
    pub click_through: Option<bool>,
    /// Whether to show window decorations (title bar, borders)
    pub decorations: Option<bool>,
}
