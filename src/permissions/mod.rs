// Temporary Permissions struct until Issue #8
#[derive(Clone, Default)]
pub struct Permissions {
    pub allowed: Vec<String>,
}

impl Permissions {
    pub fn has(&self, permission: &str) -> bool {
        self.allowed.contains(&permission.to_string()) || self.allowed.contains(&"all".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_has_specific() {
        let perms = Permissions {
            allowed: vec!["image".to_string()],
        };
        assert!(perms.has("image"));
        assert!(!perms.has("video"));
    }

    #[test]
    fn test_permission_has_all() {
        let perms = Permissions {
            allowed: vec!["all".to_string()],
        };
        assert!(perms.has("image"));
        assert!(perms.has("video"));
        assert!(perms.has("anything"));
    }

    #[test]
    fn test_permission_empty() {
        let perms = Permissions { allowed: vec![] };
        assert!(!perms.has("image"));
    }
}
