use super::WriteLinesWindow;
use super::types::{WindowHandle, WindowResponse};
use anyhow::Result;
use i_slint_backend_winit::WinitWindowAccessor;
use slint::ComponentHandle;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use tracing::{debug, info};

pub fn spawn(
    handle: WindowHandle,
    text: &str,
    font_size: f32,
    text_color: [f32; 4],
    background_color: [f32; 4],
    alignment: &str,
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
    let _ = slint::spawn_local(async move {
        if let Some(window) = window_weak.upgrade()
            && let Ok(winit_window) = window.window().winit_window().await
        {
            winit_window.set_ime_allowed(true);
            winit_window.focus_window();
            winit_window.set_resizable(false);
            winit_window.set_decorations(false);
            winit_window.set_window_icon(None);
        }
    });

    // Request focus on the text input
    window.invoke_grab_focus();

    debug!("Spawned write_lines window: {:?}", handle);
    Ok(window)
}
