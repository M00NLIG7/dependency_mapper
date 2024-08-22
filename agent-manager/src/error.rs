use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Module execution error: {0}")]
    ModuleExecution(String),

    #[error("Task join error: {0}")]
    TaskJoinError(String),

    #[error("Invalid plugin output: {0}")]
    InvalidModuleOutput(String),

    #[error("Invalid plugin input: {0}")]
    InvalidModuleInput(String),

    #[error("Invalid plugin type: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error("Module not found: {0}")]
    ModuleNotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;

