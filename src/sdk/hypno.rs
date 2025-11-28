use deno_core::op2;
use deno_core::OpState;
use std::cell::RefCell;
use std::rc::Rc;
use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;

#[op2(async)]
pub async fn op_show_hypno(
    state: Rc<RefCell<OpState>>,
    #[string] pattern: String,
) -> Result<u32, OpError> {
    {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "hypno")?;
    }
    println!("Showing hypno: {}", pattern);
    Ok(3)
}

pub const TS_SOURCE: &str = include_str!("js/hypno.ts");

deno_core::extension!(
    goon_hypno,
    ops = [op_show_hypno],
);
