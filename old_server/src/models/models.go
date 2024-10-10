package models

import (
    "context"
    "fmt"
    "github.com/neo4j/neo4j-go-driver/v5/neo4j"
)

type OS string

const (
    Linux   OS = "Linux"
    Windows OS = "Windows"
    Mac     OS = "Mac"
    Unknown OS = "Unknown"
)

type Node struct {
    ID   string `json:"id"`
    OS   string `json:"os"`
    Type string `json:"type"`
}

type Connection struct {
    ID          string `json:"id"`
    Protocol    string `json:"protocol"`
    SourcePort  int    `json:"sourcePort"`
    TargetPort  int    `json:"targetPort"`
    Description string `json:"description"`
}

type Edge struct {
    SourceID     string `json:"source"`
    ConnectionID string `json:"connection"`
    TargetID     string `json:"target"`
}

type Dependency struct {
    LocalIp     string `json:"localIp"`
    LocalOS     OS     `json:"localOS"`
    RemoteIp    string `json:"remoteIp"`
    Module      string `json:"module"`
    LocalPort   int    `json:"localPort"`
    RemotePort  int    `json:"remotePort"`
    Description string `json:"description"`
}

func AddNode(ctx context.Context, driver neo4j.DriverWithContext, node Node) (Node, error) {
    query := `
    MERGE (n:Node {id: $id})
    ON CREATE SET n.os = $os, n.type = $type
    ON MATCH SET 
        n.type = $type,
        n.os = CASE 
            WHEN n.os = 'Unknown' AND $os <> 'Unknown' THEN $os
            WHEN n.os IS NULL THEN $os
            ELSE n.os 
        END
    RETURN n.id AS id, n.os AS os, n.type AS type
    `

    result, err := neo4j.ExecuteQuery(ctx, driver, query,
        map[string]interface{}{
            "id":   node.ID,
            "os":   node.OS,
            "type": node.Type,
        },
        neo4j.EagerResultTransformer,
        neo4j.ExecuteQueryWithDatabase("neo4j"))

    if err != nil {
        return Node{}, err
    }

    if len(result.Records) == 0 {
        return Node{}, fmt.Errorf("no node created or updated")
    }

    record := result.Records[0]
    id, _ := record.Get("id")
    os, _ := record.Get("os")
    nodeType, _ := record.Get("type")

    return Node{
        ID:   id.(string),
        OS:   os.(string),
        Type: nodeType.(string),
    }, nil
}

func AddConnection(ctx context.Context, driver neo4j.DriverWithContext, conn Connection) (Connection, error) {
    query := `
    MERGE (c:Connection {id: $id})
    ON CREATE SET c.protocol = $protocol, c.sourcePort = $sourcePort, c.targetPort = $targetPort, c.description = $description
    ON MATCH SET c.protocol = $protocol, c.sourcePort = $sourcePort, c.targetPort = $targetPort, c.description = $description
    RETURN c.id AS id, c.protocol AS protocol, c.sourcePort AS sourcePort, c.targetPort AS targetPort, c.description AS description
    `

    result, err := neo4j.ExecuteQuery(ctx, driver, query,
        map[string]interface{}{
            "id":          conn.ID,
            "protocol":    conn.Protocol,
            "sourcePort":  conn.SourcePort,
            "targetPort":  conn.TargetPort,
            "description": conn.Description,
        },
        neo4j.EagerResultTransformer,
        neo4j.ExecuteQueryWithDatabase("neo4j"))

    if err != nil {
        return Connection{}, err
    }

    if len(result.Records) == 0 {
        return Connection{}, fmt.Errorf("no connection created or updated")
    }

    record := result.Records[0]
    id, _ := record.Get("id")
    protocol, _ := record.Get("protocol")
    sourcePort, _ := record.Get("sourcePort")
    targetPort, _ := record.Get("targetPort")
    description, _ := record.Get("description")

    return Connection{
        ID:          id.(string),
        Protocol:    protocol.(string),
        SourcePort:  int(sourcePort.(int64)),
        TargetPort:  int(targetPort.(int64)),
        Description: description.(string),
    }, nil
}

func AddRelationships(ctx context.Context, driver neo4j.DriverWithContext, sourceID, targetID, connectionID string) error {
    query := `
    MATCH (source:Node {id: $sourceID})
    MATCH (target:Node {id: $targetID})
    MATCH (conn:Connection {id: $connectionID})
    MERGE (source)-[:CONNECTS_TO]->(conn)-[:CONNECTS_TO]->(target)
    `

    _, err := neo4j.ExecuteQuery(ctx, driver, query,
        map[string]interface{}{
            "sourceID":     sourceID,
            "targetID":     targetID,
            "connectionID": connectionID,
        },
        neo4j.EagerResultTransformer,
        neo4j.ExecuteQueryWithDatabase("neo4j"))

    return err
}

func GetAllNodes(ctx context.Context, driver neo4j.DriverWithContext) ([]Node, error) {
    query := "MATCH (n:Node) RETURN n.id AS id, n.os AS os, n.type AS type"
    result, err := neo4j.ExecuteQuery(ctx, driver, query, nil, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
    if err != nil {
        return nil, err
    }

    var nodes []Node
    for _, record := range result.Records {
        id, _ := record.Get("id")
        os, _ := record.Get("os")
        nodeType, _ := record.Get("type")
        nodes = append(nodes, Node{
            ID:   id.(string),
            OS:   os.(string),
            Type: nodeType.(string),
        })
    }
    return nodes, nil
}

func GetAllConnections(ctx context.Context, driver neo4j.DriverWithContext) ([]Connection, error) {
    query := `
    MATCH (c:Connection)
    RETURN c.id AS id, c.protocol AS protocol, c.sourcePort AS sourcePort, c.targetPort AS targetPort, c.description AS description
    `
    result, err := neo4j.ExecuteQuery(ctx, driver, query, nil, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
    if err != nil {
        return nil, err
    }

    var connections []Connection
    for _, record := range result.Records {
        id, _ := record.Get("id")
        protocol, _ := record.Get("protocol")
        sourcePort, _ := record.Get("sourcePort")
        targetPort, _ := record.Get("targetPort")
        description, _ := record.Get("description")
        connections = append(connections, Connection{
            ID:          id.(string),
            Protocol:    protocol.(string),
            SourcePort:  int(sourcePort.(int64)),
            TargetPort:  int(targetPort.(int64)),
            Description: description.(string),
        })
    }
    return connections, nil
}

func GetAllEdges(ctx context.Context, driver neo4j.DriverWithContext) ([]Edge, error) {
    query := `
    MATCH (source:Node)-[:CONNECTS_TO]->(conn:Connection)-[:CONNECTS_TO]->(target:Node)
    RETURN source.id AS sourceId, conn.id AS connectionId, target.id AS targetId
    `
    result, err := neo4j.ExecuteQuery(ctx, driver, query, nil, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
    if err != nil {
        return nil, err
    }

    var edges []Edge
    for _, record := range result.Records {
        sourceId, _ := record.Get("sourceId")
        connectionId, _ := record.Get("connectionId")
        targetId, _ := record.Get("targetId")
        edges = append(edges, Edge{
            SourceID:     sourceId.(string),
            ConnectionID: connectionId.(string),
            TargetID:     targetId.(string),
        })
    }
    return edges, nil
}
