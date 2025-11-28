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
