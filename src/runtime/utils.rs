use crate::permissions::{Permission, PermissionChecker};
use crate::runtime::error::OpError;
use deno_core::OpState;
use deno_core::error::AnyError;
use std::str::FromStr;

pub fn check_permission(state: &mut OpState, permission_str: &str) -> Result<(), OpError> {
    let checker = state.borrow::<PermissionChecker>();
    let permission = Permission::from_str(permission_str)
        .map_err(|e| AnyError::msg(format!("Invalid permission: {}", e)))?;
    
    checker.check(permission).map_err(|e| AnyError::msg(e).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{Permission, PermissionChecker, PermissionSet};

    #[test]
    fn test_check_permission() {
        let runtime = deno_core::JsRuntime::new(Default::default());
        {
            let op_state = runtime.op_state();
            let mut state = op_state.borrow_mut();
            
            let mut set = PermissionSet::new();
            set.add(Permission::Image);
            state.put(PermissionChecker::new(set));
            
            assert!(check_permission(&mut state, "image").is_ok());
            assert!(check_permission(&mut state, "video").is_err());
            assert!(check_permission(&mut state, "invalid").is_err());
        }
    }
}
