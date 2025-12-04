use serde::Deserialize;
use std::path::PathBuf;
use ts_rs::TS;
use uuid::Uuid;

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
    /// Whether to show window decorations (title bar, borders)
    pub decorations: Option<bool>,
}

/// Unique identifier for a window.
/// You can use this handle to close the window later using `window.close(handle)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowHandle(pub Uuid);

/// Information about an active window
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub handle: WindowHandle,
    pub window_type: String,
    pub description: String,
}

/// Commands that can be sent to the window spawner
#[derive(Debug, Clone)]
pub enum WindowCommand {
    /// Get list of active windows
    GetActiveWindows(std::sync::mpsc::Sender<Vec<WindowInfo>>),
    /// Spawn a new write_lines window
    SpawnWriteLines {
        handle: WindowHandle,
        text: String,
        font_size: f32,
        text_color: [f32; 4],
        background_color: [f32; 4],
        alignment: String,
        window_options: Option<WindowOptions>,
    },
    /// Spawn a new image window
    SpawnImage {
        handle: WindowHandle,
        path: PathBuf,
        width: Option<u32>,
        height: Option<u32>,
        opacity: f32,
    },
    /// Spawn a new video window
    SpawnVideo {
        handle: WindowHandle,
        path: PathBuf,
        width: Option<u32>,
        height: Option<u32>,
        opacity: f32,
        loop_playback: bool,
        volume: f32,
    },
    /// Pause a video
    PauseVideo(WindowHandle),
    /// Resume a video
    ResumeVideo(WindowHandle),
    /// Close a specific window
    CloseWindow(WindowHandle),
    /// Close all windows
    CloseAll,
}

/// Response from window operations
#[derive(Debug, Clone)]
pub enum WindowResponse {
    /// Window was spawned successfully
    Spawned(WindowHandle),
    /// Window was closed
    Closed(WindowHandle),
    /// User submitted input from a prompt window
    PromptSubmitted { handle: WindowHandle, input: String },
    /// Error occurred
    Error(String),
}
