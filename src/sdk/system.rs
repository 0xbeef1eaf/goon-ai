use crate::gui::{WindowHandle, WindowSpawnerHandle};
use crate::runtime::error::OpError;
use deno_core::OpState;
use deno_core::op2;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

/// Closes a window by its handle ID.
///
/// You can also use the `.close()` method on the handle object returned by show functions.
///
/// @param handle - The handle ID of the window to close.
#[op2(async)]
pub async fn op_close_window(
    state: Rc<RefCell<OpState>>,
    #[string] handle: String,
) -> Result<(), OpError> {
    let window_spawner = {
        let state = state.borrow();
        state.borrow::<WindowSpawnerHandle>().clone()
    };

    let uuid = Uuid::parse_str(&handle).map_err(|e| OpError::new(&e.to_string()))?;
    window_spawner
        .close_window(WindowHandle(uuid))
        .map_err(|e| OpError::new(&e.to_string()))?;
    Ok(())
}

deno_core::extension!(goon_system, ops = [op_close_window,],);
