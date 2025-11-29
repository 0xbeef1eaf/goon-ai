use super::WallpaperSetter;
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SPI_GETDESKWALLPAPER, SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
    SystemParametersInfoW,
};

pub struct WindowsWallpaperSetter;

impl WallpaperSetter for WindowsWallpaperSetter {
    fn get_wallpaper(&self) -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            use std::ffi::OsString;
            use std::os::windows::ffi::OsStringExt;

            let mut buffer = [0u16; 260]; // MAX_PATH
            let result = unsafe {
                SystemParametersInfoW(
                    SPI_GETDESKWALLPAPER,
                    buffer.len() as u32,
                    buffer.as_mut_ptr() as *mut _,
                    0,
                )
            };

            if result == 0 {
                return Err(anyhow!("Failed to get wallpaper via SystemParametersInfoW"));
            }

            let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
            let path = OsString::from_wide(&buffer[..len]);
            Ok(PathBuf::from(path))
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(anyhow!("Not supported on this platform"))
        }
    }

    fn set_wallpaper(&self, path: &Path) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            let path_str: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

            let result = unsafe {
                SystemParametersInfoW(
                    SPI_SETDESKWALLPAPER,
                    0,
                    path_str.as_ptr() as *mut _,
                    SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
                )
            };

            if result == 0 {
                return Err(anyhow!("Failed to set wallpaper via SystemParametersInfoW"));
            }
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = path;
            Err(anyhow!("Not supported on this platform"))
        }
    }
}
