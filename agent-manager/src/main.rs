use std::path::Path;
use agent::Config;
use agent::CollectionEngine;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = Path::new("test.yaml");
    let config_str = std::fs::read_to_string(config_path)?;
    let config: Config = serde_yaml::from_str(&config_str)?;

    let mut engine = CollectionEngine::new(config);
    
    engine.run().await?;

    Ok(())
}
