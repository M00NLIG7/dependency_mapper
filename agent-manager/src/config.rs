use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct PluginConfig {
    pub name: String,
    pub command: String,
    pub args: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Config {
    pub plugins: Vec<PluginConfig>,
}
