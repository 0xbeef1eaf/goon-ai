use crate::gui::slint_controller::SlintGuiController;
use crate::gui::window_manager::{GuiInterface, WindowHandle};
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

#[op2(fast)]
pub fn op_log(state: &mut OpState, #[string] msg: String) {
    if let Some(collector) = state.try_borrow::<LogCollector>()
        && let Ok(mut logs) = collector.logs.lock()
    {
        logs.push(msg.clone());
    }
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
    state: Rc<RefCell<OpState>>,
    #[string] handle: String,
) -> Result<(), OpError> {
    let gui_controller = {
        let state = state.borrow();
        state.borrow::<Arc<SlintGuiController>>().clone()
    };

    let uuid = Uuid::parse_str(&handle).map_err(|e| OpError::new(&e.to_string()))?;
    gui_controller
        .close_window(WindowHandle(uuid))
        .map_err(|e| OpError::new(&e.to_string()))?;
    Ok(())
}

pub const TS_SOURCE: &str = include_str!("js/system.ts");

deno_core::extension!(goon_system, ops = [op_log, op_get_asset, op_close_window,],);
