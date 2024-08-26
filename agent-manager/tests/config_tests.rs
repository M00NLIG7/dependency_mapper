use agent::config::{Config, ServerConfig, AgentConfig, ModuleConfig};
use std::collections::HashMap;
use std::path::PathBuf;
use serde_json::json;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;


pub fn create_temp_script(content: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("script.sh");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "#!/bin/bash").unwrap();
    write!(file, "{}", content).unwrap();
    file.flush().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    (dir, file_path)
}

pub fn create_temp_config(content: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("config.yaml");
    let mut file = File::create(&file_path).unwrap();
    write!(file, "{}", content).unwrap();
    file.flush().unwrap();
    (dir, file_path)
}


#[test]
fn test_deserialize_valid_config() {
    let config_content = r#"
    server:
      url: "http://localhost:8000/api/v1/collect"
      timeout: 30
    agent:
      module_paths:
        - "/usr/local/lib/dep_map/modules"
        - "/usr/share/dep_map/modules"
        - "/tmp"
      log_level: "info"
    modules:
      std.modules.connection:
        description: "Description of connection module"
        interval: 15
    "#;
    let (_dir, config_file) = create_temp_config(config_content);
    
    let config: Config = serde_yaml::from_str(&std::fs::read_to_string(config_file).unwrap()).unwrap();
    
    assert_eq!(config.server, ServerConfig {
        url: "http://localhost:8000/api/v1/collect".to_string(),
        timeout: 30,
    });
    
    assert_eq!(config.agent, AgentConfig {
        module_paths: vec![
            PathBuf::from("/usr/local/lib/dep_map/modules"),
            PathBuf::from("/usr/share/dep_map/modules"),
            PathBuf::from("/tmp"),
        ],
        log_level: "info".to_string(),
    });
    
    let expected_modules = {
        let mut map = HashMap::new();
        map.insert("std.modules.connection".to_string(), ModuleConfig {
            description: Some("Description of connection module".to_string()),
            interval: 15,
            args: None,
        });
        map
    };
    
    assert_eq!(config.modules, expected_modules);
}

#[test]
fn test_deserialize_multiple_modules() {
    let config_content = r#"
    server:
      url: "http://localhost:8000/api/v1/collect"
      timeout: 30
    agent:
      module_paths:
        - "/usr/local/lib/dep_map/modules"
      log_level: "debug"
    modules:
      std.modules.connection:
        description: "Connection module"
        interval: 15
      custom.module:
        interval: 300
        args:
          key: "value"
    "#;
    let (_dir, config_file) = create_temp_config(config_content);
    
    let config: Config = serde_yaml::from_str(&std::fs::read_to_string(config_file).unwrap()).unwrap();
    
    assert_eq!(config.modules.len(), 2);
    
    assert_eq!(config.modules.get("std.modules.connection"), Some(&ModuleConfig {
        description: Some("Connection module".to_string()),
        interval: 15,
        args: None,
    }));
    
    let custom_module = config.modules.get("custom.module").unwrap();
    assert_eq!(custom_module.description, None);
    assert_eq!(custom_module.interval, 300);
    assert_eq!(custom_module.args, Some({
        let mut map = HashMap::new();
        map.insert("key".to_string(), json!("value"));
        map
    }));
}

#[test]
fn test_deserialize_empty_modules() {
    let config_content = r#"
    server:
      url: "http://localhost:8000/api/v1/collect"
      timeout: 30
    agent:
      module_paths: []
      log_level: "info"
    modules: {}
    "#;
    let (_dir, config_file) = create_temp_config(config_content);
    
    let config: Config = serde_yaml::from_str(&std::fs::read_to_string(config_file).unwrap()).unwrap();
    
    assert_eq!(config.modules.len(), 0);
}

#[test]
fn test_deserialize_invalid_config() {
    let config_content = r#"
    server:
      url: "http://localhost:8000/api/v1/collect"
    agent:
      module_paths: ["/invalid/path"]
    modules:
      invalid_module:
        invalid_field: "This should cause an error"
    "#;
    let (_dir, config_file) = create_temp_config(config_content);
    
    let result: Result<Config, serde_yaml::Error> = serde_yaml::from_str(&std::fs::read_to_string(config_file).unwrap());
    
    assert!(result.is_err());
}
