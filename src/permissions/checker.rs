use super::types::{Permission, PermissionSet};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PermissionChecker {
    permissions: Arc<PermissionSet>,
}

impl PermissionChecker {
    pub fn new(permissions: PermissionSet) -> Self {
        Self {
            permissions: Arc::new(permissions),
        }
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn check(&self, permission: Permission) -> Result<(), String> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(format!(
                "Permission denied: '{}'. This action requires the '{}' permission. \
                The pack has requested this permission, but it has not been granted. \
                To grant this permission, add '{}' to runtime.permissions in settings.yaml",
                permission, permission, permission
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checker_allow() {
        let mut set = PermissionSet::new();
        set.add(Permission::Image);
        let checker = PermissionChecker::new(set);

        assert!(checker.has_permission(Permission::Image));
        assert!(checker.check(Permission::Image).is_ok());
    }

    #[test]
    fn test_checker_deny() {
        let set = PermissionSet::new();
        let checker = PermissionChecker::new(set);

        assert!(!checker.has_permission(Permission::Video));
        assert!(checker.check(Permission::Video).is_err());
    }
}
