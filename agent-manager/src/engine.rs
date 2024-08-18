use crate::config::{Config, PluginConfig};
use crate::error::{Error, Result};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

const PLUGIN_TIMEOUT: Duration = Duration::from_secs(10); // 10 second timeout

pub struct CollectionEngine {
    plugins: Vec<Arc<PluginConfig>>,
}

impl CollectionEngine {
    pub fn new(plugins: Vec<PluginConfig>) -> Arc<Self> {
        Arc::new(CollectionEngine {
            plugins: plugins.into_iter().map(Arc::new).collect(),
        })
    }

    pub fn load_config(path: impl AsRef<Path>) -> Result<Arc<Self>> {
        let config_str = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_str)?;
        Ok(Self::new(config.plugins))
    }

    pub async fn collect_all_data(self: &Arc<Self>) -> Vec<Result<Value>> {
        let mut results = Vec::with_capacity(self.plugins.len());

        for plugin in &self.plugins {
            let plugin = Arc::clone(plugin);
            println!("Starting plugin: {}", plugin.name);
            let result = Self::run_plugin(&plugin).await;
            println!("Finished plugin: {}", plugin.name);
            results.push(result);
        }

        results
    }

    async fn run_plugin(plugin: &PluginConfig) -> Result<Value> {
        let input = serde_json::to_value(&plugin.args)?;
        println!("Running plugin '{}' with input: {:?}", plugin.name, input);

        let temp_file = NamedTempFile::new()?;
        serde_json::to_writer(&temp_file, &input)?;

        let output = Command::new(&plugin.command)
            .arg(temp_file.path())
            .output()
            .await?;

        if !output.status.success() {
            return Err(Error::PluginExecution(format!(
                "Plugin '{}' execution failed: {}",
                plugin.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let result: Value = serde_json::from_slice(&output.stdout)?;
        println!("Plugin '{}' result: {:?}", plugin.name, result);

        Ok(result)
    }
}
