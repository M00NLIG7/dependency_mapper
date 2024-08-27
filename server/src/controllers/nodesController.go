// controllers/controllers.go

package controllers

import (
    "context"
    "fmt"
    "net/http"
    "depedency-mapper-server/models"
    "github.com/gin-gonic/gin"
    "github.com/neo4j/neo4j-go-driver/v5/neo4j"
)

var driver neo4j.DriverWithContext

func InitDriver(uri, username, password string) error {
    var err error
    driver, err = neo4j.NewDriverWithContext(uri, neo4j.BasicAuth(username, password, ""))
    if err != nil {
        return err
    }
    return driver.VerifyConnectivity(context.Background())
}

func HandleDependencies(c *gin.Context) {
    var dependencies []models.Dependency
    if err := c.ShouldBindJSON(&dependencies); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid input data"})
        return
    }

    ctx := context.Background()

    for _, dep := range dependencies {
        // Add or update the local node
        localNode, err := models.AddNode(ctx, driver, models.Node{
            IP:   dep.LocalIp,
            OS:   string(dep.LocalOS),
            Type: "Local",
        })
        if err != nil {
            c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add or update local node"})
            return
        }

        // Add or update the remote node
        remoteNode, err := models.AddNode(ctx, driver, models.Node{
            IP:   dep.RemoteIp,
            OS:   string(models.Unknown),
            Type: "Remote",
        })
        if err != nil {
            c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add or update remote node"})
            return
        }

        // Add the edge between the nodes
        err = models.AddEdge(ctx, driver, localNode.IP, remoteNode.IP)
        if err != nil {
            c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add edge"})
            return
        }

        // Add the dependency details
        err = models.AddDependency(ctx, driver, dep)
        if err != nil {
            c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add dependency details"})
            return
        }
    }

    c.JSON(http.StatusOK, gin.H{"message": "Dependencies added successfully"})
}

func HandleDependency(c *gin.Context) {
    var dep models.Dependency
    if err := c.ShouldBindJSON(&dep); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid input data"})
        return
    }

    ctx := context.Background()

    // Add or update the local node
    localNode, err := models.AddNode(ctx, driver, models.Node{
        IP:   dep.LocalIp,
        OS:   string(dep.LocalOS),
        Type: "Local",
    })
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add or update local node"})
        return
    }

    // Add or update the remote node
    remoteNode, err := models.AddNode(ctx, driver, models.Node{
        IP:   dep.RemoteIp,
        OS:   string(models.Unknown),
        Type: "Remote",
    })
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add or update remote node"})
        return
    }

    // Add the edge between the nodes
    err = models.AddEdge(ctx, driver, localNode.IP, remoteNode.IP)
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add edge"})
        return
    }

    // Add the dependency details
    err = models.AddDependency(ctx, driver, dep)
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add dependency details"})
        return
    }

    c.JSON(http.StatusOK, gin.H{"message": "Dependency added successfully"})
}

func CreateOrUpdateNode(c *gin.Context) {
    var req models.Node
    if err := c.ShouldBindJSON(&req); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
        return
    }

    ctx := context.Background()

    // Add or update the node
    updatedNode, err := models.AddNode(ctx, driver, req)
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create or update node"})
        return
    }

    // Respond with the created or updated node
    c.JSON(http.StatusOK, gin.H{
        "ip":   updatedNode.IP,
        "os":   updatedNode.OS,
        "type": updatedNode.Type,
    })
}

func GetNetworkGraph(c *gin.Context) {
    ctx := context.Background()
    
    // Query to get all nodes
    nodesQuery := `
    MATCH (n:Node)
    RETURN n.ip AS ip, n.os AS os, n.type AS type
    `
    
    nodesResult, err := neo4j.ExecuteQuery(ctx, driver, nodesQuery, nil, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to retrieve nodes"})
        return
    }

    // Query to get all edges
    edgesQuery := `
    MATCH (source:Node)-[r:DEPENDS_ON]->(target:Node)
    RETURN source.ip AS sourceIp, target.ip AS targetIp
    `

    edgesResult, err := neo4j.ExecuteQuery(ctx, driver, edgesQuery, nil, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to retrieve edges"})
        return
    }

    // Process nodes
    var nodes []gin.H
    for _, record := range nodesResult.Records {
        ip, _ := record.Get("ip")
        os, _ := record.Get("os")
        nodeType, _ := record.Get("type")
        nodes = append(nodes, gin.H{
            "id": ip.(string),
            "os": os.(string),
            "type": nodeType.(string),
        })
    }

    // Process edges
    var edges []gin.H
    for _, record := range edgesResult.Records {
        sourceIp, _ := record.Get("sourceIp")
        targetIp, _ := record.Get("targetIp")
        edges = append(edges, gin.H{
            "source": sourceIp.(string),
            "target": targetIp.(string),
        })
    }

    // Send the response
    c.JSON(http.StatusOK, gin.H{
        "nodes": nodes,
        "edges": edges,
    })
}

func GetDependencies() ([]models.Dependency, error) {
    ctx := context.Background()
    
    // Query to get dependencies with all necessary information
    query := `
    MATCH (source:Node)-[r:DEPENDS_ON]->(target:Node)
    RETURN source.ip AS localIp, source.os AS localOS, target.ip AS remoteIp,
           r.module AS module, r.localPort AS localPort, r.remotePort AS remotePort,
           r.description AS description
    `
    
    result, err := neo4j.ExecuteQuery(ctx, driver, query, nil, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
    if err != nil {
        return nil, fmt.Errorf("failed to retrieve dependencies: %w", err)
    }

    var dependencies []models.Dependency
    for _, record := range result.Records {
        localIp, _ := record.Get("localIp")
        localOS, _ := record.Get("localOS")
        remoteIp, _ := record.Get("remoteIp")
        module, _ := record.Get("module")
        localPort, _ := record.Get("localPort")
        remotePort, _ := record.Get("remotePort")
        description, _ := record.Get("description")

        dependency := models.Dependency{
            LocalIp:     localIp.(string),
            LocalOS:     models.OS(localOS.(string)),
            RemoteIp:    remoteIp.(string),
            Module:      module.(string),
            LocalPort:   int(localPort.(int64)),
            RemotePort:  int(remotePort.(int64)),
            Description: description.(string),
        }
        dependencies = append(dependencies, dependency)
    }

    return dependencies, nil
}
