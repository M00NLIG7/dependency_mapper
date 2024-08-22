use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub agent: AgentConfig,
    pub modules: HashMap<String, ModuleConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AgentConfig {
    pub module_paths: Vec<PathBuf>,
    pub log_level: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModuleConfig {
    pub description: Option<String>,
    pub interval: u64,
    pub args: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub url: String,
    pub timeout: u64,
}
