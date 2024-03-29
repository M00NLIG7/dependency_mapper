# Overview

This project aims to map and visualize network dependencies, focusing on how different machines (nodes) interact and depend on each other through specific connections (edges), like LDAP. By identifying and detailing these dependencies, the project facilitates a comprehensive understanding of the network's structure and operational dynamics.

# Data Structures

To represent the network and its dependencies, the following JSON structures are proposed:

# Node Structure

Each machine in the network is represented as a node with associated attributes. The node structure includes details such as the machine's identifier, type, and any relevant properties.

```json
{
  "id": "machine1",
  "type": "server",
  "properties": {
    "ip_address": "192.168.1.1",
    "os": "Linux",
    "services": ["LDAP", "HTTP"]
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

# Data Considerations

- ## Uniqueness: Ensure each node and edge has a unique identifier to avoid ambiguities in the network graph.
- ## Scalability: Design the JSON structures with scalability in mind, allowing for the addition of new nodes, edges, and properties as the network evolves.
- ## Flexibility: Allow for the inclusion of various types of dependencies beyond LDAP, such as HTTP, database connections, etc., to provide a comprehensive network map.
- ## Security: Include security-related properties where relevant, such as encryption types and authentication methods, to aid in security analysis.
- ## Interoperability: Ensure the JSON structure is compatible with common graph databases and visualization tools to facilitate analysis and reporting.
