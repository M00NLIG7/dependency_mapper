use crate::config::{CollectedData, Config, PluginConfig};
use crate::error::{Error, Result};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::process::Command;
use tokio::task;

pub struct CollectionEngine {
    plugins: Vec<Arc<PluginConfig>>,
}

impl CollectionEngine {
    pub fn new(plugins: Vec<PluginConfig>) -> Self {
        CollectionEngine {
            plugins: plugins.into_iter().map(Arc::new).collect(),
        }
    }

    pub fn load_config(path: impl AsRef<Path>) -> Result<Self> {
        let config_str = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_str)?;
        Ok(Self::new(config.plugins))
    }

    pub async fn collect_all_data(&self) -> Vec<Result<CollectedData>> {
        let mut handles = Vec::with_capacity(self.plugins.len());

        for plugin in &self.plugins {
            let plugin = Arc::clone(plugin);
            let handle = task::spawn(async move {
                let output = Command::new(&plugin.command)
                    .args(&plugin.args)
                    .output()
                    .await?;

                if !output.status.success() {
                    return Err(Error::PluginExecution(format!(
                        "Plugin execution failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )));
                }

                let collected_data: CollectedData = serde_json::from_slice(&output.stdout)?;
                Ok(collected_data)
            });
            handles.push(handle);
        }

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            results.push(
                handle
                    .await
                    .unwrap_or_else(|e| Err(Error::TaskJoinError(e.to_string()))),
            );
        }
        results
    }
}
