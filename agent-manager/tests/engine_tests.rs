use crate::common::{create_temp_script, create_temp_config};
use agent::engine::CollectionEngine;
use agent::error::Result;

#[tokio::test]
async fn test_echo_plugin() -> Result<()> {
    let (_script_dir, script_path) = create_temp_script(r#"
        INPUT=$(cat $1)
        MESSAGE=$(echo "$INPUT" | jq -r '.name // "World"')
        echo "{\"msg\": \"Hello, $MESSAGE!\", \"failed\": false, \"changed\": false}"
    "#);

    let (_config_dir, config_path) = create_temp_config(&format!(r#"
    plugins:
      - name: echo_plugin
        command: {}
        args:
          name: "Test"
    "#, script_path.to_str().unwrap()));

    let engine = CollectionEngine::load_config(config_path)?;
    let results = engine.collect_all_data().await;

    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok(), "Error: {:?}", results[0].as_ref().unwrap_err());
    let result = results[0].as_ref().unwrap();
    assert_eq!(result["msg"].as_str().unwrap(), "Hello, Test!");
    assert_eq!(result["failed"].as_bool().unwrap(), false);
    assert_eq!(result["changed"].as_bool().unwrap(), false);

    Ok(())
}

#[tokio::test]
async fn test_math_plugin() -> Result<()> {
    let (_script_dir, script_path) = create_temp_script(r#"
        INPUT=$(cat $1)
        X=$(echo "$INPUT" | jq -r '.x')
        Y=$(echo "$INPUT" | jq -r '.y')
        SUM=$((X + Y))
        PRODUCT=$((X * Y))
        echo "{\"msg\": \"Calculation complete\", \"failed\": false, \"changed\": false, \"sum\": $SUM, \"product\": $PRODUCT}"
    "#);

    let (_config_dir, config_path) = create_temp_config(&format!(r#"
    plugins:
      - name: math_plugin
        command: {}
        args:
          x: 5
          y: 3
    "#, script_path.to_str().unwrap()));

    let engine = CollectionEngine::load_config(config_path)?;
    let results = engine.collect_all_data().await;

    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok(), "Error: {:?}", results[0].as_ref().unwrap_err());
    let result = results[0].as_ref().unwrap();
    assert_eq!(result["msg"].as_str().unwrap(), "Calculation complete");
    assert_eq!(result["failed"].as_bool().unwrap(), false);
    assert_eq!(result["changed"].as_bool().unwrap(), false);
    assert_eq!(result["sum"].as_i64().unwrap(), 8);
    assert_eq!(result["product"].as_i64().unwrap(), 15);

    Ok(())
}

#[tokio::test]
async fn test_list_files_plugin() -> Result<()> {
    let (_script_dir, script_path) = create_temp_script(r#"
        INPUT=$(cat $1)
        DIRECTORY=$(echo "$INPUT" | jq -r '.directory')
        FILES=$(ls -1 "$DIRECTORY" | jq -R -s -c 'split("\n")[:-1]')
        COUNT=$(echo "$FILES" | jq length)
        echo "{\"msg\": \"File listing complete\", \"failed\": false, \"changed\": false, \"files\": $FILES, \"count\": $COUNT}"
    "#);

    let (_config_dir, config_path) = create_temp_config(&format!(r#"
    plugins:
      - name: list_files_plugin
        command: {}
        args:
          directory: "/tmp"
    "#, script_path.to_str().unwrap()));

    let engine = CollectionEngine::load_config(config_path)?;
    let results = engine.collect_all_data().await;

    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok(), "Error: {:?}", results[0].as_ref().unwrap_err());
    let result = results[0].as_ref().unwrap();
    assert_eq!(result["msg"].as_str().unwrap(), "File listing complete");
    assert_eq!(result["failed"].as_bool().unwrap(), false);
    assert_eq!(result["changed"].as_bool().unwrap(), false);
    assert!(result["files"].is_array());
    assert!(result["count"].is_number());

    Ok(())
}

#[tokio::test]
async fn test_env_var_plugin() -> Result<()> {
    let (_script_dir, script_path) = create_temp_script(r#"
        INPUT=$(cat $1)
        VAR_NAME=$(echo "$INPUT" | jq -r '.var_name')
        VALUE=${!VAR_NAME}
        echo "{\"msg\": \"Environment variable retrieved\", \"failed\": false, \"changed\": false, \"value\": \"$VALUE\"}"
    "#);

    let (_config_dir, config_path) = create_temp_config(&format!(r#"
    plugins:
      - name: env_var_plugin
        command: {}
        args:
          var_name: "PATH"
    "#, script_path.to_str().unwrap()));

    let engine = CollectionEngine::load_config(config_path)?;
    let results = engine.collect_all_data().await;

    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok(), "Error: {:?}", results[0].as_ref().unwrap_err());
    let result = results[0].as_ref().unwrap();
    assert_eq!(result["msg"].as_str().unwrap(), "Environment variable retrieved");
    assert_eq!(result["failed"].as_bool().unwrap(), false);
    assert_eq!(result["changed"].as_bool().unwrap(), false);
    assert!(result["value"].is_string());
    assert!(!result["value"].as_str().unwrap().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_multiple_plugins() -> Result<()> {
    let (_echo_script_dir, echo_script_path) = create_temp_script(r#"
        INPUT=$(cat $1)
        MESSAGE=$(echo "$INPUT" | jq -r '.name // "World"')
        echo "{\"msg\": \"Hello, $MESSAGE!\", \"failed\": false, \"changed\": false}"
    "#);

    let (_math_script_dir, math_script_path) = create_temp_script(r#"
        INPUT=$(cat $1)
        X=$(echo "$INPUT" | jq -r '.x')
        Y=$(echo "$INPUT" | jq -r '.y')
        SUM=$((X + Y))
        PRODUCT=$((X * Y))
        echo "{\"msg\": \"Calculation complete\", \"failed\": false, \"changed\": false, \"sum\": $SUM, \"product\": $PRODUCT}"
    "#);

    let (_config_dir, config_path) = create_temp_config(&format!(r#"
    plugins:
      - name: echo_plugin
        command: {}
        args:
          name: "Test"
      - name: math_plugin
        command: {}
        args:
          x: 5
          y: 3
    "#, echo_script_path.to_str().unwrap(), math_script_path.to_str().unwrap()));

    let engine = CollectionEngine::load_config(config_path)?;
    let results = engine.collect_all_data().await;

    assert_eq!(results.len(), 2);
    assert!(results[0].is_ok(), "Error in echo plugin: {:?}", results[0].as_ref().unwrap_err());
    assert!(results[1].is_ok(), "Error in math plugin: {:?}", results[1].as_ref().unwrap_err());

    let echo_result = results[0].as_ref().unwrap();
    assert_eq!(echo_result["msg"].as_str().unwrap(), "Hello, Test!");

    let math_result = results[1].as_ref().unwrap();
    assert_eq!(math_result["sum"].as_i64().unwrap(), 8);
    assert_eq!(math_result["product"].as_i64().unwrap(), 15);

    Ok(())
}
