pub trait IPC {
    fn send(&self, message: &[u8]) -> impl std::future::Future<Output = std::io::Result<()>> + Send;
    fn receive(&self) -> impl std::future::Future<Output = std::io::Result<Vec<u8>>> + Send;
}

#[cfg(target_os = "linux")]
mod unix;

#[cfg(target_os = "windows")]
mod windows;

pub mod communicator;
