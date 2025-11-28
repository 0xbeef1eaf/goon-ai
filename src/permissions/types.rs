use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Image,
    Video,
    Audio,
    Hypno,
    Wallpaper,
    Prompt,
    Website,
}

#[derive(Debug)]
pub struct ParsePermissionError(String);

impl std::fmt::Display for ParsePermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown permission: {}", self.0)
    }
}

impl std::error::Error for ParsePermissionError {}

impl FromStr for Permission {
    type Err = ParsePermissionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "image" => Ok(Permission::Image),
            "video" => Ok(Permission::Video),
            "audio" => Ok(Permission::Audio),
            "hypno" => Ok(Permission::Hypno),
            "wallpaper" => Ok(Permission::Wallpaper),
            "prompt" => Ok(Permission::Prompt),
            "website" => Ok(Permission::Website),
            _ => Err(ParsePermissionError(s.to_string())),
        }
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::Image => write!(f, "image"),
            Permission::Video => write!(f, "video"),
            Permission::Audio => write!(f, "audio"),
            Permission::Hypno => write!(f, "hypno"),
            Permission::Wallpaper => write!(f, "wallpaper"),
            Permission::Prompt => write!(f, "prompt"),
            Permission::Website => write!(f, "website"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSet {
    pub permissions: HashSet<Permission>,
}

impl PermissionSet {
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    pub fn add(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    pub fn contains(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    pub fn union(&self, other: &PermissionSet) -> PermissionSet {
        let mut new_set = self.clone();
        for perm in &other.permissions {
            new_set.add(*perm);
        }
        new_set
    }

    pub fn intersection(&self, other: &PermissionSet) -> PermissionSet {
        let mut new_set = PermissionSet::new();
        for perm in &self.permissions {
            if other.contains(*perm) {
                new_set.add(*perm);
            }
        }
        new_set
    }

    pub fn difference(&self, other: &PermissionSet) -> PermissionSet {
        let mut new_set = PermissionSet::new();
        for perm in &self.permissions {
            if !other.contains(*perm) {
                new_set.add(*perm);
            }
        }
        new_set
    }

    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }

    pub fn len(&self) -> usize {
        self.permissions.len()
    }
}

impl From<Vec<Permission>> for PermissionSet {
    fn from(perms: Vec<Permission>) -> Self {
        let mut set = PermissionSet::new();
        for perm in perms {
            set.add(perm);
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_from_str() {
        assert_eq!(Permission::from_str("image").unwrap(), Permission::Image);
        assert_eq!(Permission::from_str("VIDEO").unwrap(), Permission::Video);
        assert_eq!(Permission::from_str("Audio").unwrap(), Permission::Audio);
        assert_eq!(Permission::from_str("hypno").unwrap(), Permission::Hypno);
        assert_eq!(
            Permission::from_str("wallpaper").unwrap(),
            Permission::Wallpaper
        );
        assert_eq!(Permission::from_str("prompt").unwrap(), Permission::Prompt);
        assert_eq!(
            Permission::from_str("website").unwrap(),
            Permission::Website
        );

        assert!(Permission::from_str("unknown").is_err());
    }

    #[test]
    fn test_permission_display() {
        assert_eq!(Permission::Image.to_string(), "image");
        assert_eq!(Permission::Video.to_string(), "video");
    }

    #[test]
    fn test_permission_set_operations() {
        let mut set1 = PermissionSet::new();
        set1.add(Permission::Image);
        set1.add(Permission::Video);

        let mut set2 = PermissionSet::new();
        set2.add(Permission::Video);
        set2.add(Permission::Audio);

        // Union
        let union = set1.union(&set2);
        assert_eq!(union.len(), 3);
        assert!(union.contains(Permission::Image));
        assert!(union.contains(Permission::Video));
        assert!(union.contains(Permission::Audio));

        // Intersection
        let intersection = set1.intersection(&set2);
        assert_eq!(intersection.len(), 1);
        assert!(intersection.contains(Permission::Video));
        assert!(!intersection.contains(Permission::Image));

        // Difference (set1 - set2)
        let diff = set1.difference(&set2);
        assert_eq!(diff.len(), 1);
        assert!(diff.contains(Permission::Image));
        assert!(!diff.contains(Permission::Video));
    }

    #[test]
    fn test_permission_set_from_vec() {
        let vec = vec![Permission::Image, Permission::Video];
        let set: PermissionSet = vec.into();
        assert_eq!(set.len(), 2);
        assert!(set.contains(Permission::Image));
        assert!(set.contains(Permission::Video));
    }

    #[test]
    fn test_permission_set_is_empty() {
        let set = PermissionSet::new();
        assert!(set.is_empty());

        let mut set = PermissionSet::new();
        set.add(Permission::Image);
        assert!(!set.is_empty());
    }
}
