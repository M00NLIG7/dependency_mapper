use agent::CollectionEngine;
use crate::common::create_temp_config_file;

mod common;

#[test]
fn test_load_valid_config() {
    let config_content = r#"
    plugins:
      - name: test_plugin
        command: echo
        args: 
          - "Hello, World!"
    "#;
    let config_file = create_temp_config_file(config_content);
    
    let engine = CollectionEngine::load_config(config_file.path().to_str().unwrap()).unwrap();
    
    assert_eq!(engine.plugins.len(), 1);
    assert_eq!(engine.plugins[0].name, "test_plugin");
    assert_eq!(engine.plugins[0].command, "echo");
    assert_eq!(engine.plugins[0].args, vec!["Hello, World!"]);
}

// Add more engine-specific tests here...
