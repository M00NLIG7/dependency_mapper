use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Plugin execution error: {0}")]
    PluginExecution(String),

    #[error("Task join error: {0}")]
    TaskJoinError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
