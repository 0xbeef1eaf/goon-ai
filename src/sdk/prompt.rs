use crate::gui::WindowSpawnerHandle;
use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
use crate::sdk::types::WindowOptions;
use deno_core::OpState;
use deno_core::op2;
use serde::Deserialize;
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, error, info};
use ts_rs::TS;

#[derive(Deserialize, Debug, Default, TS)]
#[serde(rename_all = "camelCase")]
/// Options for displaying a text prompt
pub struct PromptOptions {
    /// The text content to display in the prompt
    pub text: String,
    /// Font size in pixels
    pub font_size: Option<f32>,
    /// Text color as RGBA array [r, g, b, a] with values from 0.0 to 1.0
    pub color: Option<[f32; 4]>,
    /// Background color as RGBA array [r, g, b, a] with values from 0.0 to 1.0
    pub background: Option<[f32; 4]>,
    /// Padding around the text in pixels
    pub padding: Option<f32>,
    /// Maximum width of the text area in pixels before wrapping
    pub max_width: Option<u32>,
    /// Text alignment: "left", "center", or "right"
    pub alignment: Option<String>,
    /// Window configuration options
    pub window: Option<WindowOptions>,
    /// Duration to display the prompt in seconds, after this the window will be closed automatically
    pub duration: Option<f64>,
}

/// Displays text that the user has to repeat back to you before they can close the window.
///
/// Returns a handle ID that can be used to control the window (move, resize, close).
///
/// @param options - Optional configuration including the text to display,
///                  font settings, colors, window position, and size.
/// @returns A unique handle ID string for controlling this prompt window.
#[op2(async)]
#[string]
pub async fn op_show_prompt(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<String, OpError> {
    info!("op_show_prompt called");
    let window_spawner = {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "prompt")?;
        state.borrow::<WindowSpawnerHandle>().clone()
    };

    let opts: PromptOptions = if let Some(o) = options {
        debug!("op_show_prompt options: {:?}", o);
        serde_json::from_value(o).map_err(|e| {
            error!("Failed to parse prompt options: {}", e);
            OpError::new(&e.to_string())
        })?
    } else {
        error!("Prompt options missing");
        return Err(OpError::new("Prompt options required"));
    };

    let alignment = opts.alignment.unwrap_or_else(|| "left".to_string());
    let font_size = opts.font_size.unwrap_or(32.0);
    let text_color = opts.color.unwrap_or([1.0, 1.0, 1.0, 1.0]);
    let background_color = opts.background.unwrap_or([0.1, 0.1, 0.1, 0.95]);

    info!("Spawning prompt window via channel");
    let handle = window_spawner
        .spawn_prompt(
            opts.text,
            font_size,
            text_color,
            background_color,
            alignment,
        )
        .map_err(|e| {
            error!("Failed to spawn prompt window: {}", e);
            OpError::new(&e.to_string())
        })?;

    info!("Prompt window spawned successfully: {:?}", handle);
    Ok(handle.0.to_string())
}

deno_core::extension!(goon_prompt, ops = [op_show_prompt],);
