# Dependency Mapper (dep_map)

## Overview

Dependency Mapper (dep_map) is a modular, extensible system for mapping and analyzing dependencies in complex environments. It uses a plugin-based architecture to collect and process data from various sources, operating as a daemon that continuously collects and reports data at specified intervals.

## Installation

```bash
cargo install dep_map
```

## Configuration

### Config File Locations

dep_map looks for configuration files in the following order:

1. Path specified by the `--config` command-line option
2. `./dep_map.yaml` in the current directory
3. `$HOME/.config/dep_map/config.yaml`
4. `/etc/dep_map/config.yaml`

### Configuration Structure

```yaml
server_url: "https://your-server-url.com"
default_interval: 300  # Default interval in seconds
module_paths:
  - "/usr/local/lib/dep_map/modules"
  - "/usr/share/dep_map/modules"
  - "~/.local/share/dep_map/modules"
log_level: "info"

modules:
  - name: connections
    command: connections_module
    interval: 600  # Custom interval in seconds
    args:
      retries: 3

  - name: inventory
    command: inventory_module
    # This module will use the default interval
    args:
      include_offline: true
```

#### Global Settings

- `server_url`: URL of the server to send collected data
- `default_interval`: Default interval (in seconds) for running modules
- `module_paths`: List of directories to search for modules (in order)
- `log_level`: Global log level (debug, info, warn, error)

#### Module Configuration

- `name`: Unique identifier for the module
- `command`: Name of the module to execute (without file extension)
- `interval`: (Optional) Custom interval for this module (in seconds)
- `args`: Module-specific arguments

### Environment Variables

- `DEP_MAP_MODULE_PATH`: Additional module paths (colon-separated)
- `DEP_MAP_CONFIG`: Path to a specific config file
- `DEP_MAP_LOG_LEVEL`: Override global log level
- `DEP_MAP_SERVER_URL`: Override server URL

## Module Development

### Module Location

Modules can be placed in any directory listed in `module_paths` or `DEP_MAP_MODULE_PATH`. The recommended location for custom modules is `~/.local/share/dep_map/modules/`.

### Module Interface

Modules must adhere to the following interface:

1. Accept a single argument: path to a JSON file containing module arguments
2. Read configuration from the provided JSON file
3. Perform the module's specific functionality
4. Output results as a JSON object to stdout
5. Use exit codes to indicate success (0) or failure (non-zero)

### Example Module (Python)

```python
#!/usr/bin/env python3
import json
import sys

def main(config_path):
    with open(config_path) as f:
        config = json.load(f)
    
    # Module-specific logic here
    result = {"status": "success", "data": {...}}
    
    print(json.dumps(result))
    sys.exit(0)

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: module <config_file>")
        sys.exit(1)
    main(sys.argv[1])
```

### Example Module (Rust)

```rust
use std::env;
use std::fs;
use serde_json::{Value, json};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <config_file>", args[0]);
        std::process::exit(1);
    }

    let config: Value = serde_json::from_str(&fs::read_to_string(&args[1])?)?;

    // Module-specific logic here
    let result = json!({
        "status": "success",
        "data": { /* ... */ }
    });

    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
```

## Usage

Basic usage:

```bash
dep_map
```

With a specific config file:

```bash
dep_map --config /path/to/config.yaml
```

## Orchestrator Operation

1. dep_map loads the configuration file
2. The CollectionOrchestrator is initialized with the config
3. For each configured module:
   a. The module is scheduled to run at its specified interval
   b. When it's time to run, the module is executed with its args
   c. Output is captured, parsed as JSON, and normalized
   d. Processed data is sent to the specified server
4. This process continues indefinitely, with each module running on its own schedule

## Best Practices

1. Use semantic versioning for your modules
2. Include a `--version` flag in your modules for version checking
3. Implement proper error handling and logging in modules
4. Use typed arguments and return values within modules
5. Keep modules focused on a single responsibility
6. Use common libraries for shared functionality across modules
7. Implement unit and integration tests for modules
8. Design modules to be idempotent and tolerant of network issues

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
