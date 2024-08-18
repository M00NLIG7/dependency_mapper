use agent::config::{Config, PluginConfig};
use crate::common::create_temp_config;
use std::collections::HashMap;
use serde_json::json;

#[test]
fn test_deserialize_valid_config() {
    let config_content = r#"
    plugins:
      - name: test_plugin
        command: echo
        args:
          message: "Hello, World!"
    "#;
    let (_dir, config_file) = create_temp_config(config_content);
    
    let config: Config = serde_yaml::from_str(&std::fs::read_to_string(config_file).unwrap()).unwrap();
    
    assert_eq!(config.plugins.len(), 1);
    assert_eq!(config.plugins[0], PluginConfig {
        name: "test_plugin".to_string(),
        command: "echo".to_string(),
        args: {
            let mut map = HashMap::new();
            map.insert("message".to_string(), json!("Hello, World!"));
            map
        },
    });
}

#[test]
fn test_deserialize_multiple_plugins() {
    let config_content = r#"
    plugins:
      - name: echo_plugin
        command: echo
        args:
          message: "Hello, World!"
      - name: math_plugin
        command: ./math.sh
        args:
          x: 5
          y: 3
    "#;
    let (_dir, config_file) = create_temp_config(config_content);
    
    let config: Config = serde_yaml::from_str(&std::fs::read_to_string(config_file).unwrap()).unwrap();
    
    assert_eq!(config.plugins.len(), 2);
    
    assert_eq!(config.plugins[0], PluginConfig {
        name: "echo_plugin".to_string(),
        command: "echo".to_string(),
        args: {
            let mut map = HashMap::new();
            map.insert("message".to_string(), json!("Hello, World!"));
            map
        },
    });

    assert_eq!(config.plugins[1], PluginConfig {
        name: "math_plugin".to_string(),
        command: "./math.sh".to_string(),
        args: {
            let mut map = HashMap::new();
            map.insert("x".to_string(), json!(5));
            map.insert("y".to_string(), json!(3));
            map
        },
    });
}

#[test]
fn test_deserialize_empty_config() {
    let config_content = r#"
    plugins: []
    "#;
    let (_dir, config_file) = create_temp_config(config_content);
    
    let config: Config = serde_yaml::from_str(&std::fs::read_to_string(config_file).unwrap()).unwrap();
    
    assert_eq!(config.plugins.len(), 0);
}

#[test]
fn test_deserialize_invalid_config() {
    let config_content = r#"
    plugins:
      - name: invalid_plugin
        command: echo
        invalid_field: "This should cause an error"
    "#;
    let (_dir, config_file) = create_temp_config(config_content);
    
    let result: Result<Config, serde_yaml::Error> = serde_yaml::from_str(&std::fs::read_to_string(config_file).unwrap());
    
    assert!(result.is_err());
}
