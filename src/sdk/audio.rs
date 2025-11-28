use deno_core::op2;
use deno_core::OpState;
use std::cell::RefCell;
use std::rc::Rc;
use crate::runtime::error::OpError;
use crate::runtime::utils::check_permission;

#[op2(async)]
pub async fn op_play_audio(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<(), OpError> {
    {
        let mut state = state.borrow_mut();
        check_permission(&mut state, "audio")?;
    }
    println!("Playing audio: {}", path);
    Ok(())
}

pub const TS_SOURCE: &str = include_str!("js/audio.ts");

deno_core::extension!(
    goon_audio,
    ops = [op_play_audio],
);
