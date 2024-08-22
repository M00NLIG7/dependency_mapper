use crate::config::{Config, ModuleConfig};
use crate::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use tokio::time;

#[derive(Debug)]
pub struct CollectionEngine {
    config: Config,
    module_last_run: HashMap<String, time::Instant>,
}

impl CollectionEngine {
    pub fn new(config: Config) -> Self {
        CollectionEngine {
            config,
            module_last_run: HashMap::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            for (name, module) in &self.config.modules {
                let interval = Duration::from_secs(module.interval); let now = time::Instant::now();

                dbg!(name, &interval, &now, &self.module_last_run);
                if let Some(last_run) = self.module_last_run.get(name) {
                    if now.duration_since(*last_run) < interval {
                        continue;
                    }
                }

                let module_path = self.find_module_path(name)?;
                if let Err(e) = self.run_module(name, &module_path, module).await {
                    eprintln!("Error running module '{}': {}", name, e);
                    self.module_last_run.insert(name.clone(), now);
                } else {
                    self.module_last_run.insert(name.clone(), now);
                }
            }

            time::sleep(Duration::from_secs(1)).await;
        }
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
        Err(crate::Error::ModuleNotFound(module_name.to_string()))
    }

    async fn run_module(&self, name: &str, path: &Path, module: &ModuleConfig) -> Result<()> {
        println!("Running module '{}' from path {:?}", name, path);

        let mut command = Command::new(path);

        if let Some(args) = &module.args {
            let args_json = serde_json::to_string(args)?;
            command.arg(args_json);
        }

        let output = command.output().await?;

        if !output.status.success() {
            return Err(crate::Error::ModuleExecution(format!(
                "Module '{}' failed with status: {}",
                name, output.status
            )));
        }

        let result: Value = serde_json::from_slice(&output.stdout)?;

        // Send result to server
        self.send_to_server(&result).await?;

        Ok(())
    }

    async fn send_to_server(&self, data: &Value) -> Result<()> {
        let client = reqwest::Client::new();

        let response = client
            .post(&self.config.server.url)
            .timeout(Duration::from_secs(self.config.server.timeout))
            .json(data)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(crate::Error::ModuleExecution(format!(
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

