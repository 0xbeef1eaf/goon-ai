use super::window_manager::{GuiInterface, WindowHandle, WindowOptions};
use anyhow::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tracing::{debug, info};
use uuid::Uuid;

// Import the generated Slint modules
slint::include_modules!();

// Thread-local storage for active Slint windows (Slint is single-threaded)
thread_local! {
    static WINDOW_STATE: RefCell<HashMap<WindowHandle, Rc<PromptWindow>>> = RefCell::new(HashMap::new());
}

/// Slint GUI controller that manages windows
/// Note: This controller MUST be used from the Slint event loop thread
#[derive(Clone, Default)]
pub struct SlintGuiController;

impl SlintGuiController {
    pub fn new() -> Self {
        Self
    }

    /// Run the Slint event loop. This should be called from the main thread.
    pub fn run_event_loop() -> Result<()> {
        info!("SlintGuiController: Starting Slint event loop");
        slint::run_event_loop().map_err(|e| anyhow::anyhow!("Slint event loop error: {}", e))?;
        info!("SlintGuiController: Slint event loop ended");
        Ok(())
    }

    /// Run the Slint event loop until all windows are closed
    pub fn run_event_loop_until_quit() -> Result<()> {
        info!("SlintGuiController: Starting Slint event loop (until quit)");
        slint::run_event_loop_until_quit()
            .map_err(|e| anyhow::anyhow!("Slint event loop error: {}", e))?;
        info!("SlintGuiController: Slint event loop ended");
        Ok(())
    }
}

impl GuiInterface for SlintGuiController {
    fn create_window(&self, options: WindowOptions) -> Result<WindowHandle> {
        debug!(
            "SlintGuiController: Creating window with options: {:?}",
            options
        );

        let handle = WindowHandle(Uuid::new_v4());

        // Create the Slint window
        let window = PromptWindow::new()
            .map_err(|e| anyhow::anyhow!("Failed to create Slint window: {}", e))?;

        // Apply window options
        if let Some((width, height)) = options.size {
            window
                .window()
                .set_size(slint::PhysicalSize::new(width, height));
        }

        // Store the window
        WINDOW_STATE.with(|state| {
            state.borrow_mut().insert(handle, Rc::new(window));
        });

        // Show the window
        WINDOW_STATE.with(|state| {
            if let Some(window) = state.borrow().get(&handle) {
                window
                    .show()
                    .map_err(|e| anyhow::anyhow!("Failed to show window: {}", e))
            } else {
                Err(anyhow::anyhow!("Window not found"))
            }
        })?;

        info!(
            "SlintGuiController: Window created and shown. Handle: {:?}",
            handle
        );

        Ok(handle)
    }

    fn close_window(&self, handle: WindowHandle) -> Result<()> {
        debug!("SlintGuiController: Closing window: {:?}", handle);

        WINDOW_STATE.with(|state| {
            let mut windows = state.borrow_mut();
            if let Some(window) = windows.remove(&handle) {
                window
                    .hide()
                    .map_err(|e| anyhow::anyhow!("Failed to hide window: {}", e))?;
            }

            // Check if all windows are closed
            if windows.is_empty() {
                info!("SlintGuiController: All windows closed, quitting event loop");
                slint::quit_event_loop()
                    .map_err(|e| anyhow::anyhow!("Failed to quit event loop: {}", e))?;
            }
            Ok(())
        })
    }

    fn set_content(
        &self,
        handle: WindowHandle,
        _content: Box<dyn super::content::ContentConstructor>,
    ) -> Result<()> {
        debug!(
            "SlintGuiController: Setting content for window: {:?}",
            handle
        );

        // For now, content is set directly on the window through PromptContent
        // This method is a compatibility shim
        // TODO: Refactor to use a more generic content model

        WINDOW_STATE.with(|state| {
            if state.borrow().contains_key(&handle) {
                debug!("SlintGuiController: Content set for window: {:?}", handle);
            }
        });

        Ok(())
    }
}

/// Helper to set prompt properties on a window
pub fn set_prompt_content(
    _controller: &SlintGuiController,
    handle: WindowHandle,
    text: &str,
    font_size: f32,
    color: [f32; 4],
    background_color: Option<[f32; 4]>,
    alignment: &str,
) -> Result<()> {
    WINDOW_STATE.with(|state| {
        if let Some(window) = state.borrow().get(&handle) {
            window.set_prompt_text(text.into());
            window.set_font_size(font_size);
            window.set_text_color(slint::Color::from_rgb_u8(
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
            ));

            let bg = background_color.unwrap_or([0.0, 0.0, 0.0, 1.0]);
            window.set_background_color(slint::Color::from_rgb_u8(
                (bg[0] * 255.0) as u8,
                (bg[1] * 255.0) as u8,
                (bg[2] * 255.0) as u8,
            ));

            window.set_alignment(alignment.into());

            info!(
                "SlintGuiController: Prompt content set for handle {:?}",
                handle
            );
        }
        Ok(())
    })
}
