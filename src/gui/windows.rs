//! Window spawning and lifecycle management for goon.ai
//!
//! Uses channels to communicate between the LLM/eval thread and the Slint UI thread.
//! Each window handles its own lifecycle and cleanup.

use anyhow::Result;
use i_slint_backend_winit::WinitWindowAccessor;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, channel};
use tracing::{debug, error, info};
use uuid::Uuid;

// Import the generated Slint modules
slint::include_modules!();

/// Unique identifier for a window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowHandle(pub Uuid);

/// Commands that can be sent to the window spawner
#[derive(Debug, Clone)]
pub enum WindowCommand {
    /// Spawn a new prompt window
    SpawnPrompt {
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

/// Enum to hold different window types
enum WindowType {
    Prompt(Rc<PromptWindow>),
    Image(Rc<ImageWindow>),
}

impl WindowType {
    fn hide(&self) -> Result<(), slint::PlatformError> {
        match self {
            WindowType::Prompt(w) => w.hide(),
            WindowType::Image(w) => w.hide(),
        }
    }
}

// Thread-local storage for active windows
thread_local! {
    static WINDOWS: RefCell<HashMap<WindowHandle, WindowType>> = RefCell::new(HashMap::new());
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
                WindowCommand::SpawnPrompt {
                    handle,
                    text,
                    font_size,
                    text_color,
                    background_color,
                    alignment,
                } => {
                    if let Err(e) = self.spawn_prompt_window(
                        handle,
                        &text,
                        font_size,
                        text_color,
                        background_color,
                        &alignment,
                    ) {
                        error!("Failed to spawn prompt window: {}", e);
                        let _ = self.response_tx.send(WindowResponse::Error(e.to_string()));
                    } else {
                        let _ = self.response_tx.send(WindowResponse::Spawned(handle));
                    }
                }
                WindowCommand::SpawnImage {
                    handle,
                    path,
                    width,
                    height,
                    opacity,
                } => {
                    if let Err(e) = self.spawn_image_window(handle, &path, width, height, opacity) {
                        error!("Failed to spawn image window: {}", e);
                        let _ = self.response_tx.send(WindowResponse::Error(e.to_string()));
                    } else {
                        let _ = self.response_tx.send(WindowResponse::Spawned(handle));
                    }
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

    /// Spawn a new prompt window
    fn spawn_prompt_window(
        &self,
        handle: WindowHandle,
        text: &str,
        font_size: f32,
        text_color: [f32; 4],
        background_color: [f32; 4],
        alignment: &str,
    ) -> Result<()> {
        let window = PromptWindow::new()?;
        let window = Rc::new(window);

        // Set properties
        window.set_prompt_text(text.into());
        window.set_font_size(font_size);
        window.set_text_color(slint::Color::from_argb_u8(
            (text_color[3] * 255.0) as u8,
            (text_color[0] * 255.0) as u8,
            (text_color[1] * 255.0) as u8,
            (text_color[2] * 255.0) as u8,
        ));
        window.set_background_color(slint::Color::from_argb_u8(
            (background_color[3] * 255.0) as u8,
            (background_color[0] * 255.0) as u8,
            (background_color[1] * 255.0) as u8,
            (background_color[2] * 255.0) as u8,
        ));
        window.set_alignment(alignment.into());

        // Set up input submission callback
        let response_tx = self.response_tx.clone();
        let window_handle = handle;
        let window_weak = window.as_weak();
        window.on_input_submitted(move |input| {
            info!("Prompt submitted: {}", input);
            let _ = response_tx.send(WindowResponse::PromptSubmitted {
                handle: window_handle,
                input: input.to_string(),
            });
            // Close the window after submission
            if let Some(w) = window_weak.upgrade() {
                let _ = w.hide();
            }
        });

        // Store window
        WINDOWS.with(|windows| {
            windows
                .borrow_mut()
                .insert(handle, WindowType::Prompt(window.clone()));
        });

        // Show window
        window.show()?;

        // Configure native window properties asynchronously
        let window_weak = window.as_weak();
        let _ = slint::spawn_local(async move {
            if let Some(window) = window_weak.upgrade()
                && let Ok(winit_window) = window.window().winit_window().await
            {
                winit_window.set_ime_allowed(true);
                winit_window.focus_window();
                winit_window.set_window_level(
                    i_slint_backend_winit::winit::window::WindowLevel::AlwaysOnTop,
                );
                winit_window.set_resizable(false);
                winit_window.set_decorations(false);
                winit_window.set_window_icon(None);
            }
        });

        // Request focus on the text input
        window.invoke_grab_focus();

        debug!("Spawned prompt window: {:?}", handle);
        Ok(())
    }

    /// Spawn a new image window
    fn spawn_image_window(
        &self,
        handle: WindowHandle,
        path: &std::path::Path,
        width: Option<u32>,
        height: Option<u32>,
        opacity: f32,
    ) -> Result<()> {
        // Load the image
        let image_data = image::open(path)
            .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e))?
            .into_rgba8();

        let img_width = image_data.width();
        let img_height = image_data.height();

        // Use provided dimensions or fall back to image dimensions
        let window_width = width.unwrap_or(img_width);
        let window_height = height.unwrap_or(img_height);

        // Create Slint image from raw pixel data
        let slint_image = slint::Image::from_rgba8(slint::SharedPixelBuffer::clone_from_slice(
            image_data.as_raw(),
            img_width,
            img_height,
        ));

        let window = ImageWindow::new()?;
        let window = Rc::new(window);

        // Set properties
        window.set_source(slint_image);
        window.set_image_opacity(opacity);
        window.set_image_width(window_width as i32);
        window.set_image_height(window_height as i32);

        // Store window
        WINDOWS.with(|windows| {
            windows
                .borrow_mut()
                .insert(handle, WindowType::Image(window.clone()));
        });

        // Show window
        window.show()?;

        // Configure native window properties asynchronously
        let window_weak = window.as_weak();
        let _ = slint::spawn_local(async move {
            if let Some(window) = window_weak.upgrade()
                && let Ok(winit_window) = window.window().winit_window().await
            {
                winit_window.set_window_level(
                    i_slint_backend_winit::winit::window::WindowLevel::AlwaysOnTop,
                );
                winit_window.set_resizable(false);
                winit_window.set_decorations(false);
                winit_window.set_window_icon(None);
            }
        });

        debug!("Spawned image window: {:?}", handle);
        Ok(())
    }

    /// Close a specific window
    fn close_window(&self, handle: WindowHandle) {
        WINDOWS.with(|windows| {
            if let Some(window) = windows.borrow_mut().remove(&handle) {
                let _ = window.hide();
                debug!("Closed window: {:?}", handle);
            }
        });
    }

    /// Close all windows
    fn close_all_windows(&self) {
        WINDOWS.with(|windows| {
            let mut w = windows.borrow_mut();
            for (handle, window) in w.drain() {
                let _ = window.hide();
                debug!("Closed window: {:?}", handle);
            }
        });
    }
}

/// Handle for sending commands to the window spawner from other threads
/// This is Clone + Send + Sync safe as it only contains the sender channel
#[derive(Clone)]
pub struct WindowSpawnerHandle {
    command_tx: Sender<WindowCommand>,
}

impl WindowSpawnerHandle {
    /// Spawn a new prompt window
    pub fn spawn_prompt(
        &self,
        text: String,
        font_size: f32,
        text_color: [f32; 4],
        background_color: [f32; 4],
        alignment: String,
    ) -> Result<WindowHandle> {
        let handle = WindowHandle(Uuid::new_v4());
        self.command_tx
            .send(WindowCommand::SpawnPrompt {
                handle,
                text,
                font_size,
                text_color,
                background_color,
                alignment,
            })
            .map_err(|e| anyhow::anyhow!("Failed to send spawn command: {}", e))?;
        Ok(handle)
    }

    /// Spawn a new image window
    pub fn spawn_image(
        &self,
        path: PathBuf,
        width: Option<u32>,
        height: Option<u32>,
        opacity: f32,
    ) -> Result<WindowHandle> {
        let handle = WindowHandle(Uuid::new_v4());
        self.command_tx
            .send(WindowCommand::SpawnImage {
                handle,
                path,
                width,
                height,
                opacity,
            })
            .map_err(|e| anyhow::anyhow!("Failed to send spawn image command: {}", e))?;
        Ok(handle)
    }

    /// Close a specific window
    pub fn close_window(&self, handle: WindowHandle) -> Result<()> {
        self.command_tx
            .send(WindowCommand::CloseWindow(handle))
            .map_err(|e| anyhow::anyhow!("Failed to send close command: {}", e))
    }

    /// Close all windows
    pub fn close_all(&self) -> Result<()> {
        self.command_tx
            .send(WindowCommand::CloseAll)
            .map_err(|e| anyhow::anyhow!("Failed to send close all command: {}", e))
    }
}

/// Run the Slint event loop with periodic command processing
pub fn run_event_loop(spawner: WindowSpawner) -> Result<()> {
    info!("Starting Slint event loop with window spawner");

    // Create a timer to periodically process commands
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(16), // ~60fps
        move || {
            spawner.process_commands();
        },
    );

    slint::run_event_loop()?;
    Ok(())
}
