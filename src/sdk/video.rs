use deno_core::op2;
use deno_core::OpState;
use std::cell::RefCell;
use std::rc::Rc;
use serde_json;
use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;

#[op2(async)]
pub async fn op_show_video(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
    #[serde] options: Option<serde_json::Value>,
) -> Result<u32, OpError> {
    {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "video")?;
    }
    println!("Showing video: {} with options: {:?}", path, options);
    Ok(2)
}

pub const TS_SOURCE: &str = include_str!("js/video.ts");

deno_core::extension!(
    goon_video,
    ops = [op_show_video],
);
