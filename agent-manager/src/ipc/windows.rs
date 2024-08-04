use super::IPC;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_named_pipes::NamedPipeClientOptions;
use tokio_named_pipes::NamedPipeServerOptions;
use std::io::Result;

pub struct NamedPipeIPC {
    pub pipe_name: String,
}

impl IPC for NamedPipeIPC {
    fn send(&self, message: &[u8]) -> Result<()> {
        let mut client = NamedPipeClientOptions::new()
            .open(&self.pipe_name)?;
        client.write_all(message)?;
        Ok(())
    }

    fn receive(&self) -> Result<Vec<u8>> {
        let mut server = NamedPipeServerOptions::new()
            .create(&self.pipe_name)?;
        let mut buffer = Vec::new();
        server.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

