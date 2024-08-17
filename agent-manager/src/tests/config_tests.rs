use agent::config::{Config, PluginConfig};
use crate::common::create_temp_config_file;

mod common;

#[test]
fn test_deserialize_valid_config() {
    let config_content = r#"
    plugins:
      - name: test_plugin
        command: echo
        args: 
          - "Hello, World!"
    "#;
    let config_file = create_temp_config_file(config_content);
    
    let config: Config = serde_yaml::from_str(&std::fs::read_to_string(config_file.path().to_str().unwrap()).unwrap()).unwrap();
    
    assert_eq!(config.plugins.len(), 1);
    assert_eq!(config.plugins[0], PluginConfig {
        name: "test_plugin".to_string(),
        command: "echo".to_string(),
        args: vec!["Hello, World!".to_string()],
    });
}

// Add more config-specific tests here...
