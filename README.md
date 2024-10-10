# Network Dependency Mapping and Visualization

## Overview

This project aims to map and visualize network dependencies, focusing on how different machines (nodes) interact and depend on each other through specific connections (edges), like LDAP. By identifying and detailing these dependencies, the project facilitates a comprehensive understanding of the network's structure and operational dynamics.

## Data Structures

### Node Structure

Nodes represent machines or dependencies in the network. They are defined using the following Go struct:

```go
type Node struct {
    ID   string `json:"id"`
    OS   string `json:"os"`
    Type string `json:"type"`
}
```

### Connection Structure

Connections represent the links between nodes, defined as:

```go
type Connection struct {
    ID          string `json:"id"`
    Protocol    string `json:"protocol"`
    SourcePort  int    `json:"sourcePort"`
    TargetPort  int    `json:"targetPort"`
    Description string `json:"description"`
}
```

### Edge Structure

Edges represent the relationships between nodes and connections:

```go
type Edge struct {
    SourceID     string `json:"source"`
    ConnectionID string `json:"connection"`
    TargetID     string `json:"target"`
}
```

### Dependency Structure

Dependencies provide additional details about the connections between nodes:

```go
type Dependency struct {
    LocalIp     string `json:"localIp"`
    LocalOS     OS     `json:"localOS"`
    RemoteIp    string `json:"remoteIp"`
    Module      string `json:"module"`
    LocalPort   int    `json:"localPort"`
    RemotePort  int    `json:"remotePort"`
    Description string `json:"description"`
}
```

## Program Architecture

The project follows a distributed architecture:

1. **Local Machine**:
   - Rust Agent Manager
     - Reads YAML configurations for all local agents
     - Schedules and manages the lifecycle of agents
     - Monitors agent health and restarts if necessary
     - Handles logging and error reporting
   - Multi-language Agents (Python, Go, Node.js)
     - Implement specific data collection logic
     - Post data directly to the central server

2. **Central Server**:
   - Go Server
     - Receives data from agents via HTTP/HTTPS
     - Manages the Neo4j database for storing network topology
   - Web Server
     - Backend: Go REST API
       - Handles data processing and retrieval from Neo4j
       - Provides endpoints for the frontend to consume
     - Frontend: Next.js
       - Offers a responsive and interactive user interface
       - Visualizes network dependencies and allows for data exploration

## Database Operations

The project uses Neo4j as the backend database. Key operations include:

- Adding nodes and connections
- Creating relationships between nodes
- Retrieving all nodes, connections, and edges

## Data Considerations

- **Uniqueness**: Each node and edge has a unique identifier to avoid ambiguities in the network graph.
- **Scalability**: The data structures are designed with scalability in mind, allowing for the addition of new nodes, edges, and properties as the network evolves.
- **Flexibility**: The system allows for various types of dependencies beyond LDAP, such as HTTP, database connections, etc., to provide a comprehensive network map.
- **Security**: Security-related properties are included where relevant, such as encryption types and authentication methods, to aid in security analysis.
- **Interoperability**: The data structure is compatible with common graph databases and visualization tools to facilitate analysis and reporting.

## TODO

- Implement PIVOT MAP functionality
- Develop TIMELINE feature

## Getting Started

TODO

## Contributing

TODO

## License

This project is licensed under the GNU General Public License v3.0 (GPLv3). This license ensures that the software remains open source and that any modifications or derivative works are also distributed under the same license terms.

Key points of the GPLv3:
- You are free to use, modify, and distribute this software.
- If you distribute modified versions, you must make your modifications available under the GPLv3.
- You must include the original copyright notice and license text with any distribution.

For the full license text, see the [LICENSE](LICENSE) file in the project repository or visit [https://www.gnu.org/licenses/gpl-3.0.en.html](https://www.gnu.org/licenses/gpl-3.0.en.html).
