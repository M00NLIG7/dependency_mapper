use agent::{CollectionEngine, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = CollectionEngine::load_config("plugins.yaml")?;
    
    let results = engine.collect_all_data().await;
    for result in results {
        match result {
            Ok(data) => println!("Collected from {}: {}", data.source, data.data),
            Err(e) => eprintln!("Error collecting data: {}", e),
        }
    }

    Ok(())
}
