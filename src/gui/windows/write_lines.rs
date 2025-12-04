use super::WriteLinesWindow;
use super::types::{WindowHandle, WindowOptions, WindowResponse};
use anyhow::Result;
use i_slint_backend_winit::WinitWindowAccessor;
use slint::ComponentHandle;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use tracing::{debug, info};

#[allow(clippy::too_many_arguments)]
pub fn spawn(
    handle: WindowHandle,
    text: &str,
    font_size: f32,
    text_color: [f32; 4],
    background_color: [f32; 4],
    alignment: &str,
    window_options: Option<WindowOptions>,
    response_tx: Sender<WindowResponse>,
) -> Result<Rc<WriteLinesWindow>> {
    let window = WriteLinesWindow::new()?;
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
    let expected_text = text.to_string();
    let window_handle = handle;
    let window_weak = window.as_weak();
    window.on_input_submitted(move |input| {
        let input_str = input.to_string();
        info!("Prompt submitted: {}", input_str);

        if let Some(w) = window_weak.upgrade() {
            if input_str == expected_text {
                let _ = response_tx.send(WindowResponse::PromptSubmitted {
                    handle: window_handle,
                    input: input_str,
                });
                // Close the window after submission
                let _ = w.hide();
            } else {
                // Clear input if incorrect
                w.set_user_input("".into());
            }
        }
    });

    // Prevent closing via OS controls
    window
        .window()
        .on_close_requested(|| slint::CloseRequestResponse::KeepWindowShown);

    // Show window
    window.show()?;

    // Configure native window properties asynchronously
    let window_weak = window.as_weak();
    let options = window_options.clone();
    let _ = slint::spawn_local(async move {
        if let Some(window) = window_weak.upgrade()
            && let Ok(winit_window) = window.window().winit_window().await
        {
            winit_window.set_ime_allowed(true);
            winit_window.focus_window();

            if let Some(opts) = options {
                if let Some(always_on_top) = opts.always_on_top {
                    winit_window.set_window_level(if always_on_top {
                        winit::window::WindowLevel::AlwaysOnTop
                    } else {
                        winit::window::WindowLevel::Normal
                    });
                }

                if let Some(decorations) = opts.decorations {
                    winit_window.set_decorations(decorations);
                } else {
                    winit_window.set_decorations(false);
                }

                if let Some(pos) = opts.position {
                    winit_window
                        .set_outer_position(winit::dpi::PhysicalPosition::new(pos.x, pos.y));
                }

                if let Some(size) = opts.size {
                    let _ = winit_window
                        .request_inner_size(winit::dpi::PhysicalSize::new(size.width, size.height));
                }

                if let Some(_opacity) = opts.opacity {
                    // Slint handles opacity via window background color usually, but winit might have transparent
                    winit_window.set_transparent(true);
                    // Opacity is already handled by the window background color alpha channel in spawn()
                }
            } else {
                winit_window.set_resizable(false);
                winit_window.set_decorations(false);
            }

            winit_window.set_window_icon(None);
        }
    });

    // Request focus on the text input
    window.invoke_grab_focus();

    debug!("Spawned write_lines window: {:?}", handle);
    Ok(window)
}
