use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
use deno_core::OpState;
use deno_core::op2;
use serde::Deserialize;
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{error, info};
use ts_rs::TS;

#[derive(Deserialize, Debug, Default, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PromptOptions {
    pub text: String,
    pub title: Option<String>,
}

/// Show a prompt dialog using a native input dialog
#[op2(async)]
#[string]
pub async fn op_show_prompt(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<String, OpError> {
    info!("op_show_prompt called");

    {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "prompt")?;
    }

    let opts: PromptOptions = if let Some(o) = options {
        serde_json::from_value(o).map_err(|e| {
            error!("Failed to parse prompt options: {}", e);
            OpError::new(&e.to_string())
        })?
    } else {
        error!("Prompt options missing");
        return Err(OpError::new("Prompt options required"));
    };

    // Show native input dialog
    let title = opts.title.unwrap_or_else(|| "Enter Text".to_string());
    let message = format!("Type: {}", opts.text);

    let result = rfd::AsyncMessageDialog::new()
        .set_title(&title)
        .set_description(&message)
        .set_buttons(rfd::MessageButtons::OkCancel)
        .show()
        .await;

    if result == rfd::MessageDialogResult::Ok {
        info!("Prompt accepted");
        Ok("accepted".to_string())
    } else {
        info!("Prompt cancelled");
        Ok("cancelled".to_string())
    }
}

pub const TS_SOURCE: &str = include_str!("js/prompt.ts");

deno_core::extension!(goon_prompt, ops = [op_show_prompt],);
