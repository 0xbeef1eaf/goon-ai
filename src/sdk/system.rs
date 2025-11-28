use deno_core::op2;
use deno_core::OpState;
use std::cell::RefCell;
use std::rc::Rc;
use crate::runtime::error::OpError;

#[op2(fast)]
pub fn op_log(#[string] msg: String) {
    println!("[JS Log]: {}", msg);
}

#[op2(async)]
#[string]
pub async fn op_get_asset(
    _state: Rc<RefCell<OpState>>,
    #[string] tag: String,
) -> Result<String, OpError> {
    // Internal op, might not need explicit permission or uses a system permission
    println!("Getting asset for tag: {}", tag);
    Ok(format!("/path/to/asset/{}", tag))
}

#[op2(async)]
pub async fn op_close_window(
    _state: Rc<RefCell<OpState>>,
    handle: u32,
) -> Result<(), OpError> {
    println!("Closing window: {}", handle);
    Ok(())
}

pub const TS_SOURCE: &str = include_str!("js/system.ts");

deno_core::extension!(
    goon_system,
    ops = [
        op_log,
        op_get_asset,
        op_close_window,
    ],
);
