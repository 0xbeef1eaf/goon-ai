use anyhow::Result;
use std::path::{Path, PathBuf};

pub trait WallpaperSetter {
    fn set_wallpaper(&self, path: &Path) -> Result<()>;
    fn get_wallpaper(&self) -> Result<PathBuf>;
}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxWallpaperSetter as PlatformWallpaperSetter;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsWallpaperSetter as PlatformWallpaperSetter;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacOsWallpaperSetter as PlatformWallpaperSetter;

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
pub struct PlatformWallpaperSetter;

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
impl WallpaperSetter for PlatformWallpaperSetter {
    fn set_wallpaper(&self, _path: &Path) -> Result<()> {
        Err(anyhow::anyhow!(
            "Wallpaper setting not supported on this platform"
        ))
    }

    fn get_wallpaper(&self) -> Result<PathBuf> {
        Err(anyhow::anyhow!(
            "Wallpaper getting not supported on this platform"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_setter_implements_trait() {
        let setter = PlatformWallpaperSetter;
        // Just check if it compiles and we can call the method (even if it fails)
        // We pass a dummy path
        let _ = setter.set_wallpaper(Path::new("dummy"));
    }
}
