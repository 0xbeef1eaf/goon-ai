use anyhow::Result;
use std::path::PathBuf;

pub struct MpvDownloader;

impl MpvDownloader {
    pub async fn ensure_libmpv() -> Result<PathBuf> {
        if let Some(path) = Self::find_libmpv() {
            return Ok(path);
        }

        Self::download_libmpv().await
    }

    fn find_libmpv() -> Option<PathBuf> {
        // Check common locations
        #[cfg(target_os = "linux")]
        {
            let paths = vec![
                "/usr/lib/libmpv.so",
                "/usr/lib/libmpv.so.1",
                "/usr/lib/libmpv.so.2",
                "/usr/lib/x86_64-linux-gnu/libmpv.so",
                "/usr/lib/x86_64-linux-gnu/libmpv.so.1",
                "/usr/lib/x86_64-linux-gnu/libmpv.so.2",
                "/usr/local/lib/libmpv.so",
            ];
            for p in paths {
                let path = PathBuf::from(p);
                if path.exists() {
                    return Some(path);
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Check local directory
            if Path::new("mpv-2.dll").exists() {
                return Some(PathBuf::from("mpv-2.dll"));
            }
            if Path::new("libmpv-2.dll").exists() {
                return Some(PathBuf::from("libmpv-2.dll"));
            }
        }

        None
    }

    async fn download_libmpv() -> Result<PathBuf> {
        println!("Downloading libmpv...");

        #[cfg(target_os = "linux")]
        {
            anyhow::bail!(
                "On Linux, please install libmpv using your package manager (e.g., apt install libmpv-dev or libmpv2)"
            );
        }

        #[cfg(target_os = "windows")]
        {
            // TODO: Implement actual download from SourceForge
            anyhow::bail!(
                "Auto-download for Windows not fully implemented yet. Please place mpv-2.dll in the application directory."
            );
        }

        #[cfg(target_os = "macos")]
        {
            anyhow::bail!("On macOS, please install using Homebrew: brew install mpv");
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            anyhow::bail!("Unsupported platform for libmpv");
        }
    }
}
