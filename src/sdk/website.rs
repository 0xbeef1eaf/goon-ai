use deno_core::op2;
use deno_core::OpState;
use std::cell::RefCell;
use std::rc::Rc;
use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;

#[op2(async)]
pub async fn op_open_website(
    state: Rc<RefCell<OpState>>,
    #[string] url: String,
) -> Result<(), OpError> {
    {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "website")?;
    }
    println!("Opening website: {}", url);
    Ok(())
}

pub const TS_SOURCE: &str = include_str!("js/website.ts");

deno_core::extension!(
    goon_website,
    ops = [op_open_website],
);
