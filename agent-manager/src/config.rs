use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CollectedData {
    pub source: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PluginConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub plugins: Vec<PluginConfig>,
}
