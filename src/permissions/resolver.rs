use super::types::PermissionSet;

pub struct PermissionResolver;

impl PermissionResolver {
    pub fn resolve(
        pack_permissions: &PermissionSet,
        user_permissions: &PermissionSet,
    ) -> PermissionSet {
        // The active permissions are those that are BOTH requested by the pack AND granted by the user.
        // This ensures security (user must grant) and efficiency (only enable what pack needs).
        pack_permissions.intersection(user_permissions)
    }

    pub fn find_missing(
        pack_permissions: &PermissionSet,
        user_permissions: &PermissionSet,
    ) -> PermissionSet {
        // Permissions requested by the pack but NOT granted by the user.
        pack_permissions.difference(user_permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::Permission;

    #[test]
    fn test_resolve_intersection() {
        let mut pack = PermissionSet::new();
        pack.add(Permission::Image);
        pack.add(Permission::Video);

        let mut user = PermissionSet::new();
        user.add(Permission::Image);
        user.add(Permission::Audio);

        let resolved = PermissionResolver::resolve(&pack, &user);

        assert!(resolved.contains(Permission::Image));
        assert!(!resolved.contains(Permission::Video)); // Not in user
        assert!(!resolved.contains(Permission::Audio)); // Not in pack
    }

    #[test]
    fn test_find_missing() {
        let mut pack = PermissionSet::new();
        pack.add(Permission::Image);
        pack.add(Permission::Video);

        let mut user = PermissionSet::new();
        user.add(Permission::Image);

        let missing = PermissionResolver::find_missing(&pack, &user);

        assert!(!missing.contains(Permission::Image));
        assert!(missing.contains(Permission::Video));
    }
}
