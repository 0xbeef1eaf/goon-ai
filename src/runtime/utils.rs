use crate::permissions::{Permission, PermissionChecker};
use crate::runtime::error::OpError;
use deno_core::OpState;
use deno_core::error::AnyError;

pub fn check_permission(state: &mut OpState, permission: Permission) -> Result<(), OpError> {
    let checker = state.borrow::<PermissionChecker>();
    checker
        .check(permission)
        .map_err(|e| AnyError::msg(e).into())
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

            let checker = PermissionChecker::new(set);
            state.put(checker);
        }

        {
            let op_state = runtime.op_state();
            let mut state = op_state.borrow_mut();

            // Should pass
            assert!(check_permission(&mut state, Permission::Image).is_ok());

            // Should fail
            assert!(check_permission(&mut state, Permission::Video).is_err());
        }
    }
}
