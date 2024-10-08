use crate::config::{Config, ModuleConfig};
use crate::Error;
use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::time::Instant;
use tempfile::NamedTempFile;
use tokio::sync::mpsc;
use tokio::time;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(serialize = "PascalCase", deserialize = "snake_case"))]
pub struct Dependency {
    pub local_ip: String,
    pub local_os: String,
    pub remote_ip: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub module: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct CollectionEngine {
    config: Config,
    module_last_run: HashMap<String, Instant>,
}

impl PartialEq for CollectionEngine {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
    }
}

impl CollectionEngine {
    pub fn new(config: Config) -> Self {
        CollectionEngine {
            config,
            module_last_run: HashMap::new(),
        }
    }

    pub async fn run(&mut self, mut shutdown_rx: mpsc::Receiver<()>) -> Result<()> {
        println!("Starting engine...");
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    println!("Shutdown signal received. Gracefully shutting down...");
                    break;
                }
                _ = time::sleep(Duration::from_secs(1)) => {
                    self.run_iteration().await?;
                }
            }
        }
        Ok(())
    }

    async fn run_iteration(&mut self) -> Result<()> {
        let now = Instant::now();
        for (name, module) in &self.config.modules {
            let interval = Duration::from_secs(module.interval);
            if let Some(last_run) = self.module_last_run.get(name) {
                if now.duration_since(*last_run) < interval {
                    continue;
                }
            }

            if let Ok(module_path) = self.find_module_path(name) {
                if let Err(e) = self.run_module(name, &module_path, module).await {
                    eprintln!("Error running module '{}': {}", name, e);
                }
                self.module_last_run.insert(name.clone(), now);
            }
        }
        Ok(())
    }

    fn find_module_path(&self, module_name: &str) -> Result<PathBuf> {
        let sanitized_module_name = sanitize_module_name(module_name)?;
        let module_path = sanitized_module_name.replace(".", "/");
        for base_path in &self.config.agent.module_paths {
            let full_path = base_path.join(&module_path);
            if full_path.exists() {
                return Ok(full_path);
            }
        }
        Err(Error::ModuleNotFound(module_name.to_string()))
    }

    async fn run_module(&self, name: &str, path: &Path, module: &ModuleConfig) -> Result<()> {
        let mut command = tokio::process::Command::new(path);

        let temp_file = if let Some(args) = &module.args {
            let mut file = NamedTempFile::new()?;
            let args_json = serde_json::to_string(args)?;
            println!("Writing args to file: {}", args_json);
            file.write_all(args_json.as_bytes())?;
            Some(file)
        } else {
            None
        };

        if let Some(file) = &temp_file {
            command.env("ARGS_FILE", file.path());
        }

        let output = command.output().await?;

        // Clean up the temporary file
        if let Some(file) = temp_file {
            file.close()?;
        }

        if !output.status.success() {
            return Err(Error::ModuleExecution(format!(
                "Module '{}' failed with status: {}",
                name, output.status
            )));
        }

        let result: Value = serde_json::from_slice(&output.stdout)?;
        let data: Vec<Dependency> = match result.get("dependencies") {
            Some(Value::Array(arr)) => serde_json::from_value(Value::Array(arr.to_vec()))
                .unwrap_or_else(|e| {
                    eprintln!("Error deserializing dependencies: {:?}", e);
                    Vec::new()
                }),
            _ => {
                eprintln!("'dependencies' field is missing or not an array");
                Vec::new()
            }
        };

        // Send result to server
        self.send_to_server(&data).await?;

        Ok(())
    }

    async fn send_to_server(&self, data: &Vec<Dependency>) -> Result<()> {
        let client = reqwest::Client::new();

        let response = client
            .post(&self.config.server.url)
            .timeout(Duration::from_secs(self.config.server.timeout))
            .json(data)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::ModuleExecution(format!(
                "Server responded with status: {}",
                response.status()
            )));
        }

        Ok(())
    }
}

fn sanitize_module_name(name: &str) -> Result<String> {
    // Allow only alphanumeric characters, dots, and underscores
    let sanitized: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '_')
        .collect();

    // Ensure the sanitized name is not empty and doesn't start or end with a dot
    if sanitized.is_empty() || sanitized.starts_with('.') || sanitized.ends_with('.') {
        return Err(crate::Error::InvalidModuleName(name.to_string()));
    }

    // Prevent consecutive dots
    if sanitized.contains("..") {
        return Err(crate::Error::InvalidModuleName(name.to_string()));
    }

    Ok(sanitized)
}
