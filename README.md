# Overview

This project aims to map and visualize network dependencies, focusing on how different machines (nodes) interact and depend on each other through specific connections (edges), like LDAP. By identifying and detailing these dependencies, the project facilitates a comprehensive understanding of the network's structure and operational dynamics.

# Data Structures

To represent the network and its dependencies, the following JSON structures are proposed:

# Node Structure

Each machine in the network is represented as a node with associated attributes. The node structure includes details such as the machine's identifier, type, and any relevant properties.

```json

// Server
{
  "id": "machine1",
  "type": "server",
  "properties": {
    "ip_address": "192.168.1.1",
    "hostname": "bingus.local",
    "os": "Linux",
    "dependencies": ["LDAP", "HTTP"]
  }
}

// Dependency
{
    "id": "LDAP_1",
    "parent": "machine1",
    "type": "LDAP",
    "properties": {
       "dependants": ["192.168.1.2"],
    }
}
```

# Edge Structure

Dependencies between machines are represented as edges, indicating the direction and nature of the dependency. Each edge specifies the source and target nodes, along with the dependency type and any additional information.

```json
{
  "source": "machine1",
  "target": "machine2",
  "type": "LDAP",
  "properties": {
    "port": 389,
    "encryption": "SSL",
    "status": "active"
  }
}
```

# Graph Structure

The graph structure encapsulates the entire network, comprising a collection of nodes and edges. This structure enables a holistic view of the network dependencies.

```json
{
  "nodes": [
    {
      /* Node 1 JSON */
    },
    {
      /* Node 2 JSON */
    }
    // Additional nodes
  ],
  "edges": [
    {
      /* Edge 1 JSON */
    },
    {
      /* Edge 2 JSON */
    }
    // Additional edges
  ]
}
```

# Program Architecture

Local Machine
```
+-----------------------------------+
|   +-----------------------------+ |
|   |    Rust Agent Manager       | |
|   |  +-----------------------+  | |
|   |  | Agent Configurations  |  | |
|   |  | (YAML)                |  | |
|   |  +-----------------------+  | |
|   |           |                 | |
|   |  +--------v--------+        | |
|   |  | Agent Scheduler |        | |
|   |  +--------+--------+        | |
|   |           |                 | |
|   +-----------------------------+ |
|             |   |   |             |
|    +--------v-+ | +-v----------+  |
|    |Python    | | |Go Agent    |  |
|    |Agent     | | |            |  |
|    +----------+ | +------------+  |
|      +----------v----------+      |
|      |   Node.js Agent     |      |
|      +---------------------+      |
+-----------------------------------+
              |
              | HTTP/HTTPS
              v
+-----------------------------------+
|        Go Server                  |
+-----------------------------------+
```
Rust Agent Manager:

Reads YAML configurations for all local agents
Schedules and manages the lifecycle of agents
Monitors agent health and restarts if necessary
Handles logging and error reporting


Agent Configurations (YAML):

Define each agent's properties, schedule, and data collection parameters


Multi-language Agents:

Implement specific data collection logic
Post data directly to your existing server
Can be simple scripts or more complex programs

# Data Considerations

- ## Uniqueness: Ensure each node and edge has a unique identifier to avoid ambiguities in the network graph.
- ## Scalability: Design the JSON structures with scalability in mind, allowing for the addition of new nodes, edges, and properties as the network evolves.
- ## Flexibility: Allow for the inclusion of various types of dependencies beyond LDAP, such as HTTP, database connections, etc., to provide a comprehensive network map.
- ## Security: Include security-related properties where relevant, such as encryption types and authentication methods, to aid in security analysis.
- ## Interoperability: Ensure the JSON structure is compatible with common graph databases and visualization tools to facilitate analysis and reporting.

## TODO 
PIVOT MAP
