use crate::assets::registry::AssetRegistry;
use crate::assets::selector::AssetSelector;
use crate::assets::types::Asset;
use crate::config::pack::Mood;
use crate::gui::window_manager::GuiInterface;
use crate::media::image::animation::Animation;
use crate::media::image::loader::load_image;
use crate::media::image::renderer::ImageContent;
use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
use crate::sdk::types::WindowOptions;
use deno_core::OpState;
use deno_core::op2;
use serde::Deserialize;
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use ts_rs::TS;

#[derive(Deserialize, Debug, Default, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
/// Options for displaying an image
pub struct ImageOptions {
    /// Tags to filter the images by
    pub tags: Option<Vec<String>>,
    /// Duration to display the image in seconds
    pub duration: Option<u64>,
    #[serde(flatten)]
    pub window: WindowOptions,
}

#[op2(async)]
#[string]
pub async fn op_show_image(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<String, OpError> {
    let (gui_controller, registry, mood) = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "image")?;
        let gui = state.borrow::<Arc<dyn GuiInterface>>().clone();
        let registry = state.borrow::<Arc<AssetRegistry>>().clone();
        let mood = state.borrow::<Mood>().clone();
        (gui, registry, mood)
    };

    let opts: ImageOptions = if let Some(o) = options {
        serde_json::from_value(o).map_err(|e| OpError::new(&e.to_string()))?
    } else {
        ImageOptions::default()
    };

    let tags = opts.tags.unwrap_or_default();
    let selector = AssetSelector::new(&registry);

    let asset = selector
        .select_image(&mood, &tags)
        .ok_or_else(|| OpError::new("No image found matching tags"))?;

    let path = match asset {
        Asset::Image(img) => &img.path,
        _ => return Err(OpError::new("Selected asset is not an image")),
    };

    let path_str = path.to_str().unwrap();
    let (content, width, height) = if path_str.to_lowercase().ends_with(".gif") {
        if let Ok(anim) = Animation::load(path) {
            let w = anim.frames[0].buffer.width();
            let h = anim.frames[0].buffer.height();
            (
                ImageContent {
                    image: None,
                    animation: Some(anim),
                },
                w,
                h,
            )
        } else {
            let img = load_image(path_str).map_err(|e| OpError::new(&e.to_string()))?;
            (
                ImageContent {
                    image: Some(img.clone()),
                    animation: None,
                },
                img.width(),
                img.height(),
            )
        }
    } else {
        let img = load_image(path_str).map_err(|e| OpError::new(&e.to_string()))?;
        (
            ImageContent {
                image: Some(img.clone()),
                animation: None,
            },
            img.width(),
            img.height(),
        )
    };

    let size = opts
        .window
        .size
        .map(|s| (s.width, s.height))
        .unwrap_or((width, height));

    let window_opts = crate::gui::window_manager::WindowOptions {
        size: Some(size),
        opacity: opts.window.opacity.unwrap_or(1.0),
        always_on_top: opts.window.always_on_top.unwrap_or(false),
        click_through: opts.window.click_through.unwrap_or(false),
        position: opts.window.position.map(|p| (p.x, p.y)),
        decorations: false,
        timeout: opts.duration.map(std::time::Duration::from_secs),
        ..Default::default()
    };

    let handle = gui_controller
        .create_window(window_opts)
        .map_err(|e| OpError::new(&e.to_string()))?;

    gui_controller
        .set_content(handle, Box::new(content))
        .map_err(|e| OpError::new(&e.to_string()))?;

    Ok(handle.0.to_string())
}

pub const TS_SOURCE: &str = include_str!("js/image.ts");

deno_core::extension!(goon_image, ops = [op_show_image],);
