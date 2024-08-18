use agent::engine::CollectionEngine;
use agent::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Loading configuration...");
    let engine = CollectionEngine::load_config("plugins.yaml")?;
    println!("Configuration loaded successfully");
    
    println!("Starting data collection...");
    let results = engine.collect_all_data().await;
    println!("Data collection completed");

    for (index, result) in results.iter().enumerate() {
        match result {
            Ok(data) => println!("Plugin {}: Collected data: {:?}", index, data),
            Err(e) => eprintln!("Plugin {}: Error collecting data: {}", index, e),
        }
    }

    Ok(())
}
