use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
use crate::sdk::image::ImageOptions;
use deno_core::OpState;
use deno_core::op2;
use serde::Deserialize;
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;
use ts_rs::TS;

#[derive(Deserialize, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PromptOptions {
    text: String,
    image: Option<ImageOptions>,
}

#[op2(async)]
pub async fn op_show_prompt(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<u32, OpError> {
    {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "prompt")?;
    }

    let opts: PromptOptions = if let Some(o) = options {
        serde_json::from_value(o).map_err(|e| OpError::new(&e.to_string()))?
    } else {
        return Err(OpError::new("Prompt options required"));
    };

    println!(
        "Showing prompt: {} with image options: {:?}",
        opts.text, opts.image
    );
    Ok(4)
}

pub const TS_SOURCE: &str = include_str!("js/prompt.ts");

deno_core::extension!(goon_prompt, ops = [op_show_prompt],);
