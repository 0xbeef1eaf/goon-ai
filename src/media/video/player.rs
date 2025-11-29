use super::ipc::IpcClient;
use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::time::{Duration, sleep};
use uuid::Uuid;

pub struct VideoPlayer {
    process: Child,
    ipc: Option<IpcClient>,
    socket_path: PathBuf,
}

impl VideoPlayer {
    pub async fn spawn(
        file_path: PathBuf,
        options: &crate::sdk::video::VideoOptions,
    ) -> Result<Self> {
        let socket_id = Uuid::new_v4();
        let socket_path = std::env::temp_dir().join(format!("mpv-socket-{}", socket_id));

        let mut cmd = Command::new("mpv");
        cmd.arg(&file_path)
            .arg(format!("--input-ipc-server={}", socket_path.display()))
            .arg("--idle")
            .arg("--keep-open")
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        if let Some(vol) = options.volume {
            cmd.arg(format!("--volume={}", vol * 100.0));
        }

        if options.loop_.unwrap_or(false) {
            cmd.arg("--loop");
        }

        // Window options
        if let Some(size) = &options.window.size {
            cmd.arg(format!("--geometry={}x{}", size.width, size.height));
        }

        if let Some(pos) = &options.window.position {
            cmd.arg(format!("--geometry=+{}+{}", pos.x, pos.y));
        }

        if options.window.always_on_top.unwrap_or(false) {
            cmd.arg("--ontop");
        }

        if !options.window.decorations.unwrap_or(true) {
            cmd.arg("--no-border");
        }

        let process = cmd.spawn()?;

        // Wait for socket to be created
        let mut ipc = None;
        for _ in 0..20 {
            if let Ok(client) = IpcClient::connect(socket_path.clone()).await {
                ipc = Some(client);
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }

        Ok(Self {
            process,
            ipc,
            socket_path,
        })
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(ipc) = &mut self.ipc {
            let _ = ipc.send_command(vec![json!("quit")]).await;
        } else {
            let _ = self.process.kill().await;
        }
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<()> {
        if let Some(ipc) = &mut self.ipc {
            ipc.send_command(vec![json!("set_property"), json!("pause"), json!(true)])
                .await?;
        }
        Ok(())
    }

    pub async fn resume(&mut self) -> Result<()> {
        if let Some(ipc) = &mut self.ipc {
            ipc.send_command(vec![json!("set_property"), json!("pause"), json!(false)])
                .await?;
        }
        Ok(())
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        // Try to clean up socket
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
