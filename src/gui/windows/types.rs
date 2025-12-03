use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for a window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowHandle(pub Uuid);

/// Commands that can be sent to the window spawner
#[derive(Debug, Clone)]
pub enum WindowCommand {
    /// Spawn a new write_lines window
    SpawnWriteLines {
        handle: WindowHandle,
        text: String,
        font_size: f32,
        text_color: [f32; 4],
        background_color: [f32; 4],
        alignment: String,
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
