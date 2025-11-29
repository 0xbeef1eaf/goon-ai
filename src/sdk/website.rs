use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
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
pub struct WebsiteOptions {
    url: String,
}

#[op2(async)]
pub async fn op_open_website(
    state: Rc<RefCell<OpState>>,
    #[serde] options: Option<serde_json::Value>,
) -> Result<(), OpError> {
    {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "website")?;
    }

    let opts: WebsiteOptions = if let Some(o) = options {
        serde_json::from_value(o).map_err(|e| OpError::new(&e.to_string()))?
    } else {
        return Err(OpError::new("Website options required"));
    };

    println!("Opening website: {}", opts.url);
    Ok(())
}

pub const TS_SOURCE: &str = include_str!("js/website.ts");

deno_core::extension!(goon_website, ops = [op_open_website],);
