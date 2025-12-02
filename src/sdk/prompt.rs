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
