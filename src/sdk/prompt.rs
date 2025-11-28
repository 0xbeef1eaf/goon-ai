use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;
use deno_core::OpState;
use deno_core::op2;
use std::cell::RefCell;
use std::rc::Rc;

#[op2(async)]
pub async fn op_show_prompt(
    state: Rc<RefCell<OpState>>,
    #[string] text: String,
) -> Result<u32, OpError> {
    {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "prompt")?;
    }
    println!("Showing prompt: {}", text);
    Ok(4)
}

pub const TS_SOURCE: &str = include_str!("js/prompt.ts");

deno_core::extension!(goon_prompt, ops = [op_show_prompt],);
