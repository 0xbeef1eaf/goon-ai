use crate::gui::window_manager::{GuiController, WindowOptions};
use crate::media::image::loader::load_image;
use crate::media::image::renderer::ImageContent;
use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
use deno_core::OpState;
use deno_core::op2;
use serde::Deserialize;
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct ImageOptions {
    duration: Option<u64>,
    opacity: Option<f32>,
    position: Option<Position>,
    always_on_top: Option<bool>,
    click_through: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[op2(async)]
#[string]
pub async fn op_show_image(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
    #[serde] options: Option<serde_json::Value>,
) -> Result<String, OpError> {
    let gui_controller = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "image")?;
        state.borrow::<GuiController>().clone()
    };

    let opts: ImageOptions = if let Some(o) = options {
        serde_json::from_value(o).map_err(|e| OpError::new(&e.to_string()))?
    } else {
        ImageOptions::default()
    };

    // Load image
    let img = load_image(&path).map_err(|e| OpError::new(&e.to_string()))?;
    let width = img.width();
    let height = img.height();

    let window_opts = WindowOptions {
        size: Some((width, height)),
        opacity: opts.opacity.unwrap_or(1.0),
        always_on_top: opts.always_on_top.unwrap_or(false),
        click_through: opts.click_through.unwrap_or(false),
        position: opts.position.map(|p| (p.x, p.y)),
        decorations: false,
        timeout: opts.duration.map(std::time::Duration::from_secs),
        ..Default::default()
    };

    let handle = gui_controller
        .create_window(window_opts)
        .map_err(|e| OpError::new(&e.to_string()))?;

    let content = ImageContent {
        image: Some(img),
        animation: None,
    };

    gui_controller
        .set_content(handle, Box::new(content))
        .map_err(|e| OpError::new(&e.to_string()))?;

    Ok(handle.0.to_string())
}

pub const TS_SOURCE: &str = include_str!("js/image.ts");

deno_core::extension!(goon_image, ops = [op_show_image],);
