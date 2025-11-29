use crate::gui::window_manager::GuiInterface;
use crate::media::prompt::renderer::PromptContent;
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
pub struct PromptOptions {
    pub text: String,
    pub font_size: Option<f32>,
    pub color: Option<[f32; 4]>,
    pub background: Option<[f32; 4]>,
    pub padding: Option<f32>,
    pub max_width: Option<u32>,
    pub alignment: Option<String>, // "left", "center", "right"
    #[serde(flatten)]
    pub window: WindowOptions,
    pub duration: Option<f64>,
}

#[op2(async)]
#[string]
pub async fn op_show_prompt(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<String, OpError> {
    let gui_controller = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "prompt")?;
        state.borrow::<Arc<dyn GuiInterface>>().clone()
    };

    let opts: PromptOptions = if let Some(o) = options {
        serde_json::from_value(o).map_err(|e| OpError::new(&e.to_string()))?
    } else {
        return Err(OpError::new("Prompt options required"));
    };

    let alignment = match opts.alignment.as_deref() {
        Some("center") => glyphon::cosmic_text::Align::Center,
        Some("right") => glyphon::cosmic_text::Align::End,
        _ => glyphon::cosmic_text::Align::Left,
    };

    let content = PromptContent {
        text: opts.text.clone(),
        font_size: opts.font_size.unwrap_or(32.0),
        color: opts.color.unwrap_or([1.0, 1.0, 1.0, 1.0]),
        background_color: opts.background,
        max_width: opts.max_width,
        alignment,
    };

    let window_opts = crate::gui::window_manager::WindowOptions {
        size: opts
            .window
            .size
            .map(|s| (s.width, s.height))
            .or(Some((800, 600))), // Default size
        opacity: opts.window.opacity.unwrap_or(1.0),
        always_on_top: opts.window.always_on_top.unwrap_or(false),
        click_through: opts.window.click_through.unwrap_or(false),
        position: opts.window.position.map(|p| (p.x, p.y)),
        decorations: opts.window.decorations.unwrap_or(false), // Default to no decorations for prompts
        timeout: opts.duration.map(std::time::Duration::from_secs_f64),
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

pub const TS_SOURCE: &str = include_str!("js/prompt.ts");

deno_core::extension!(goon_prompt, ops = [op_show_prompt],);
