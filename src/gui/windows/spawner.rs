use super::image;
use super::types::{WindowCommand, WindowHandle, WindowResponse};
use super::video::{self, VideoState};
use super::write_lines;
use super::{ImageWindow, WriteLinesWindow};
use anyhow::Result;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, channel};
use tracing::{error, info};

// Thread-local storage for active windows
thread_local! {
    static WINDOWS: RefCell<HashMap<WindowHandle, WindowType>> = RefCell::new(HashMap::new());
}

/// Enum to hold different window types
enum WindowType {
    WriteLines(Rc<WriteLinesWindow>),
    Image(Rc<ImageWindow>),
    Video(VideoState),
}

impl WindowType {
    fn hide(&self) -> Result<(), slint::PlatformError> {
        match self {
            WindowType::WriteLines(w) => w.hide(),
            WindowType::Image(w) => w.hide(),
            WindowType::Video(state) => state.window.hide(),
        }
    }
}

/// Handle to send commands to the window spawner
#[derive(Clone)]
pub struct WindowSpawnerHandle {
    pub command_tx: Sender<WindowCommand>,
}

impl WindowSpawnerHandle {
    pub fn send(&self, command: WindowCommand) -> Result<()> {
        self.command_tx
            .send(command)
            .map_err(|e| anyhow::anyhow!("Failed to send window command: {}", e))
    }

    pub fn spawn_write_lines(
        &self,
        text: String,
        font_size: f32,
        text_color: [f32; 4],
        background_color: [f32; 4],
        alignment: String,
    ) -> Result<WindowHandle> {
        let handle = WindowHandle(uuid::Uuid::new_v4());
        self.send(WindowCommand::SpawnWriteLines {
            handle,
            text,
            font_size,
            text_color,
            background_color,
            alignment,
        })?;
        Ok(handle)
    }

    pub fn spawn_image(
        &self,
        path: std::path::PathBuf,
        width: Option<u32>,
        height: Option<u32>,
        opacity: f32,
    ) -> Result<WindowHandle> {
        let handle = WindowHandle(uuid::Uuid::new_v4());
        self.send(WindowCommand::SpawnImage {
            handle,
            path,
            width,
            height,
            opacity,
        })?;
        Ok(handle)
    }

    pub fn spawn_video(
        &self,
        path: std::path::PathBuf,
        width: Option<u32>,
        height: Option<u32>,
        opacity: f32,
        loop_playback: bool,
        volume: f32,
    ) -> Result<WindowHandle> {
        let handle = WindowHandle(uuid::Uuid::new_v4());
        self.send(WindowCommand::SpawnVideo {
            handle,
            path,
            width,
            height,
            opacity,
            loop_playback,
            volume,
        })?;
        Ok(handle)
    }

    pub fn pause_video(&self, handle: WindowHandle) -> Result<()> {
        self.send(WindowCommand::PauseVideo(handle))
    }

    pub fn resume_video(&self, handle: WindowHandle) -> Result<()> {
        self.send(WindowCommand::ResumeVideo(handle))
    }

    pub fn close_window(&self, handle: WindowHandle) -> Result<()> {
        self.send(WindowCommand::CloseWindow(handle))
    }
}

/// Window spawner that processes commands on the Slint UI thread
pub struct WindowSpawner {
    command_rx: Receiver<WindowCommand>,
    response_tx: Sender<WindowResponse>,
}

impl WindowSpawner {
    /// Create a new window spawner channel pair
    pub fn create() -> (WindowSpawnerHandle, Self) {
        let (command_tx, command_rx) = channel();
        let (response_tx, _response_rx) = channel();

        let handle = WindowSpawnerHandle { command_tx };

        let spawner = Self {
            command_rx,
            response_tx,
        };

        (handle, spawner)
    }

    /// Process pending commands (call this from the Slint event loop)
    pub fn process_commands(&self) {
        while let Ok(cmd) = self.command_rx.try_recv() {
            match cmd {
                WindowCommand::SpawnWriteLines {
                    handle,
                    text,
                    font_size,
                    text_color,
                    background_color,
                    alignment,
                } => {
                    match write_lines::spawn(
                        handle,
                        &text,
                        font_size,
                        text_color,
                        background_color,
                        &alignment,
                        self.response_tx.clone(),
                    ) {
                        Ok(window) => {
                            WINDOWS.with(|windows| {
                                windows
                                    .borrow_mut()
                                    .insert(handle, WindowType::WriteLines(window));
                            });
                            let _ = self.response_tx.send(WindowResponse::Spawned(handle));
                        }
                        Err(e) => {
                            error!("Failed to spawn write_lines window: {}", e);
                            let _ = self.response_tx.send(WindowResponse::Error(e.to_string()));
                        }
                    }
                }
                WindowCommand::SpawnImage {
                    handle,
                    path,
                    width,
                    height,
                    opacity,
                } => match image::spawn(handle, &path, width, height, opacity) {
                    Ok(window) => {
                        WINDOWS.with(|windows| {
                            windows
                                .borrow_mut()
                                .insert(handle, WindowType::Image(window));
                        });
                        let _ = self.response_tx.send(WindowResponse::Spawned(handle));
                    }
                    Err(e) => {
                        error!("Failed to spawn image window: {}", e);
                        let _ = self.response_tx.send(WindowResponse::Error(e.to_string()));
                    }
                },
                WindowCommand::SpawnVideo {
                    handle,
                    path,
                    width,
                    height,
                    opacity,
                    loop_playback: _,
                    volume: _,
                } => match video::spawn(handle, &path, width, height, opacity) {
                    Ok(state) => {
                        WINDOWS.with(|windows| {
                            windows
                                .borrow_mut()
                                .insert(handle, WindowType::Video(state));
                        });
                        let _ = self.response_tx.send(WindowResponse::Spawned(handle));
                    }
                    Err(e) => {
                        error!("Failed to spawn video window: {}", e);
                        let _ = self.response_tx.send(WindowResponse::Error(e.to_string()));
                    }
                },
                WindowCommand::PauseVideo(handle) => {
                    self.pause_video(handle);
                }
                WindowCommand::ResumeVideo(handle) => {
                    self.resume_video(handle);
                }
                WindowCommand::CloseWindow(handle) => {
                    self.close_window(handle);
                    let _ = self.response_tx.send(WindowResponse::Closed(handle));
                }
                WindowCommand::CloseAll => {
                    self.close_all_windows();
                }
            }
        }
    }

    fn pause_video(&self, handle: WindowHandle) {
        WINDOWS.with(|windows| {
            if let Some(WindowType::Video(state)) = windows.borrow().get(&handle)
                && let Ok(mut player) = state.player.lock()
            {
                player.pause();
            }
        });
    }

    fn resume_video(&self, handle: WindowHandle) {
        WINDOWS.with(|windows| {
            if let Some(WindowType::Video(state)) = windows.borrow().get(&handle)
                && let Ok(mut player) = state.player.lock()
            {
                player.resume();
            }
        });
    }

    fn close_window(&self, handle: WindowHandle) {
        WINDOWS.with(|windows| {
            if let Some(window_type) = windows.borrow_mut().remove(&handle) {
                let _ = window_type.hide();
            }
        });
    }

    fn close_all_windows(&self) {
        WINDOWS.with(|windows| {
            let mut windows = windows.borrow_mut();
            for (_, window_type) in windows.drain() {
                let _ = window_type.hide();
            }
        });
    }
}

/// Run the Slint event loop
pub fn run_event_loop(spawner: WindowSpawner) -> Result<()> {
    info!("Starting Slint event loop with window spawner");

    // Create a timer to poll for commands
    let timer = slint::Timer::default();
    let spawner = Rc::new(spawner);

    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(16),
        move || {
            spawner.process_commands();
        },
    );

    slint::run_event_loop()?;
    Ok(())
}
