use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

#[derive(Serialize)]
struct Command {
    command: Vec<Value>,
}

pub struct IpcClient {
    stream: UnixStream,
}

impl IpcClient {
    pub async fn connect(path: PathBuf) -> Result<Self> {
        let stream = UnixStream::connect(path).await?;
        Ok(Self { stream })
    }

    pub async fn send_command(&mut self, args: Vec<Value>) -> Result<()> {
        let cmd = Command { command: args };
        let json = serde_json::to_string(&cmd)?;
        self.stream.write_all(json.as_bytes()).await?;
        self.stream.write_all(b"\n").await?;
        Ok(())
    }
}
