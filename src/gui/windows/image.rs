use super::ImageWindow;
use super::types::WindowHandle;
use anyhow::Result;
use i_slint_backend_winit::WinitWindowAccessor;
use slint::ComponentHandle;
use std::rc::Rc;
use tracing::debug;

pub fn spawn(
    handle: WindowHandle,
    path: &std::path::Path,
    width: Option<u32>,
    height: Option<u32>,
    opacity: f32,
) -> Result<Rc<ImageWindow>> {
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

    // Show window
    window.show()?;

    // Configure native window properties asynchronously
    let window_weak = window.as_weak();
    let _ = slint::spawn_local(async move {
        if let Some(window) = window_weak.upgrade()
            && let Ok(winit_window) = window.window().winit_window().await
        {
            winit_window
                .set_window_level(i_slint_backend_winit::winit::window::WindowLevel::AlwaysOnTop);
            winit_window.set_resizable(false);
            winit_window.set_decorations(false);
            winit_window.set_window_icon(None);
        }
    });

    debug!("Spawned image window: {:?}", handle);
    Ok(window)
}
