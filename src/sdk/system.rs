use crate::gui::{WindowHandle, WindowSpawnerHandle};
use crate::runtime::error::OpError;
use deno_core::OpState;
use deno_core::op2;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct LogCollector {
    pub logs: Arc<Mutex<Vec<String>>>,
}

/// Logs a message to the console.
///
/// @param msg - The message to log.
#[op2(fast)]
pub fn op_log(state: &mut OpState, #[string] msg: String) {
    if let Some(collector) = state.try_borrow::<LogCollector>()
        && let Ok(mut logs) = collector.logs.lock()
    {
        logs.push(msg.clone());
    }
    println!("[JS Log]: {}", msg);
}

/// Gets an asset path by tag.
///
/// This is an internal operation used by other SDK modules.
///
/// @param tag - The tag to search for.
/// @returns The path to the selected asset.
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

/// Closes a window by its handle ID.
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

deno_core::extension!(goon_system, ops = [op_log, op_get_asset, op_close_window,],);
