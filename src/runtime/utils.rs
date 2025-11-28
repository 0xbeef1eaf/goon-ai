use crate::permissions::Permissions;
use crate::runtime::error::OpError;
use deno_core::OpState;
use deno_core::error::AnyError;

pub fn check_permission(state: &mut OpState, permission: &str) -> Result<(), OpError> {
    let perms = state.borrow::<Permissions>();
    if !perms.has(permission) {
        return Err(AnyError::msg(format!("Permission denied: {}", permission)).into());
    }
    Ok(())
}
