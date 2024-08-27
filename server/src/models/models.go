// models/models.go

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
    IP   string `json:"ip"`
    OS   string `json:"os"`
    Type string `json:"type"`
}

type Dependency struct {
    Module      string `json:"module"`
    LocalIp     string `json:"localIp"`
    LocalPort   int    `json:"localPort"`
    RemoteIp    string `json:"remoteIp"`
    RemotePort  int    `json:"remotePort"`
    Description string `json:"description"`
    LocalOS     OS     `json:"localOS"`
}

type Edge struct {
    SourceIP string `json:"sourceIP"`
    TargetIP string `json:"targetIP"`
}

func AddNode(ctx context.Context, driver neo4j.DriverWithContext, node Node) (Node, error) {
    query := `
    MERGE (n:Node {ip: $ip})
    ON CREATE SET n.os = $os, n.type = $type
    ON MATCH SET 
        n.type = $type,
        n.os = CASE 
            WHEN n.os = 'Unknown' AND $os <> 'Unknown' THEN $os
            WHEN n.os IS NULL THEN $os
            ELSE n.os 
        END
    RETURN n.ip AS ip, n.os AS os, n.type AS type
    `

    result, err := neo4j.ExecuteQuery(ctx, driver, query,
        map[string]interface{}{
            "ip":   node.IP,
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
    ip, _ := record.Get("ip")
    os, _ := record.Get("os")
    nodeType, _ := record.Get("type")

    return Node{
        IP:   ip.(string),
        OS:   os.(string),
        Type: nodeType.(string),
    }, nil
}

func AddDependency(ctx context.Context, driver neo4j.DriverWithContext, dep Dependency) error {
    query := `
    MATCH (source:Node {ip: $localIp})-[r:DEPENDS_ON]->(target:Node {ip: $remoteIp})
    SET r.module = $module,
        r.localPort = $localPort,
        r.remotePort = $remotePort,
        r.description = $description
    RETURN r
    `

    _, err := neo4j.ExecuteQuery(ctx, driver, query,
        map[string]interface{}{
            "localIp":     dep.LocalIp,
            "remoteIp":    dep.RemoteIp,
            "module":      dep.Module,
            "localPort":   dep.LocalPort,
            "remotePort":  dep.RemotePort,
            "description": dep.Description,
        },
        neo4j.EagerResultTransformer,
        neo4j.ExecuteQueryWithDatabase("neo4j"))

    return err
}

func GetAllNodes(ctx context.Context, driver neo4j.DriverWithContext) ([]Node, error) {
    query := "MATCH (n:Node) RETURN n.ip AS ip, n.os AS os, n.type AS type"
    result, err := neo4j.ExecuteQuery(ctx, driver, query, nil, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
    if err != nil {
        return nil, err
    }

    var nodes []Node
    for _, record := range result.Records {
        ip, _ := record.Get("ip")
        os, _ := record.Get("os")
        nodeType, _ := record.Get("type")
        nodes = append(nodes, Node{
            IP:   ip.(string),
            OS:   os.(string),
            Type: nodeType.(string),
        })
    }
    return nodes, nil
}

func GetAllEdges(ctx context.Context, driver neo4j.DriverWithContext) ([]Edge, error) {
    query := `
    MATCH (source:Node)-[:DEPENDS_ON]->(target:Node)
    RETURN source.ip AS sourceIP, target.ip AS targetIP
    `

    result, err := neo4j.ExecuteQuery(ctx, driver, query, nil, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
    if err != nil {
        return nil, err
    }

    var edges []Edge
    for _, record := range result.Records {
        sourceIP, _ := record.Get("sourceIP")
        targetIP, _ := record.Get("targetIP")

        edges = append(edges, Edge{
            SourceIP: sourceIP.(string),
            TargetIP: targetIP.(string),
        })
    }

    return edges, nil
}

func AddEdge(ctx context.Context, driver neo4j.DriverWithContext, sourceIP, targetIP string) error {
    query := `
    MATCH (source:Node {ip: $sourceIP})
    MATCH (target:Node {ip: $targetIP})
    MERGE (source)-[r:DEPENDS_ON]->(target)
    RETURN r
    `

    _, err := neo4j.ExecuteQuery(ctx, driver, query,
        map[string]interface{}{
            "sourceIP": sourceIP,
            "targetIP": targetIP,
        },
        neo4j.EagerResultTransformer,
        neo4j.ExecuteQueryWithDatabase("neo4j"))

    return err
}

