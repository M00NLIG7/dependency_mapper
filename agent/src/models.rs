use std::net::Ipv4Addr;
use std::path::PathBuf;

pub struct Dependency {
    host: String,
    port: u16,
}

pub struct Config {
    pub(crate) server: Ipv4Addr,
    pub(crate) port: u16,
    pub(crate) log_file: PathBuf,
    pub(crate) log_level: u8,
    pub(crate) modules: Vec<String>,
}
