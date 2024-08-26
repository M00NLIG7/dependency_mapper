use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub agent: AgentConfig,
    pub modules: HashMap<String, ModuleConfig>,
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.server == other.server && self.agent == other.agent && self.modules == other.modules
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AgentConfig {
    pub module_paths: Vec<PathBuf>,
    pub log_level: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ModuleConfig {
    pub description: Option<String>,
    pub interval: u64,
    pub args: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ServerConfig {
    pub url: String,
    pub timeout: u64,
}
