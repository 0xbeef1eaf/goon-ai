use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
use deno_core::OpState;
use deno_core::op2;
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;

#[op2(async)]
pub async fn op_show_image(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
    #[serde] options: Option<serde_json::Value>,
) -> Result<u32, OpError> {
    {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "image")?;
    }
    println!("Showing image: {} with options: {:?}", path, options);
    // Mock window handle
    Ok(1)
}

pub const TS_SOURCE: &str = include_str!("js/image.ts");

deno_core::extension!(goon_image, ops = [op_show_image],);
