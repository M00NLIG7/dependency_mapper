use super::IPC;
use tokio::net::{UnixStream, UnixListener};
use std::path::Path;

pub struct UnixSocketIPC {
    pub socket_path: String,
}

impl IPC for UnixSocketIPC {
    async fn send(&self, message: &[u8]) -> std::io::Result<()> {
        let stream = UnixStream::connect(&self.socket_path).await?;
        stream.try_write(message)?;
        Ok(())
    }

    async fn receive(&self) -> std::io::Result<Vec<u8>> {
        if Path::new(&self.socket_path).exists() {
            std::fs::remove_file(&self.socket_path)?; // Clean up if already exists
        }
        let listener = UnixListener::bind(&self.socket_path)?;
        let (stream, _) = listener.accept().await?;
        let mut buffer = Vec::new();
        stream.try_read(&mut buffer)?;
        Ok(buffer)
    }
}

