// controllers/controllers.go

package controllers

import (
	"context"
	"dependency-mapper/internal/models"
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j"
	"net/http"
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
			ID:   dep.LocalIp,
			OS:   string(dep.LocalOS),
			Type: "Local",
		})
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add or update local node"})
			return
		}

		// Add or update the remote node
		remoteNode, err := models.AddNode(ctx, driver, models.Node{
			ID:   dep.RemoteIp,
			OS:   string(models.Unknown),
			Type: "Remote",
		})
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add or update remote node"})
			return
		}

		// Create a unique identifier for the connection
		connectionID := fmt.Sprintf("%s-%s-%s-%d-%d", dep.LocalIp, dep.RemoteIp, dep.Module, dep.LocalPort, dep.RemotePort)

		// Add or update the connection
		conn, err := models.AddConnection(ctx, driver, models.Connection{
			ID:          connectionID,
			Protocol:    dep.Module,
			SourcePort:  dep.LocalPort,
			TargetPort:  dep.RemotePort,
			Description: dep.Description,
		})
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add or update connection"})
			return
		}

		// Add or update relationships between nodes and connection
		err = models.AddRelationships(ctx, driver, localNode.ID, remoteNode.ID, conn.ID)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add relationships"})
			return
		}
	}

	c.JSON(http.StatusOK, gin.H{"message": "Dependencies processed successfully"})
}

/*
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

*/

func GetNetworkGraph(c *gin.Context) {
	ctx := context.Background()

	// Get all nodes
	nodes, err := models.GetAllNodes(ctx, driver)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to retrieve nodes"})
		return
	}

	// Get all connections
	connections, err := models.GetAllConnections(ctx, driver)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to retrieve connections"})
		return
	}

	// Get all edges (relationships between nodes and connections)
	edges, err := models.GetAllEdges(ctx, driver)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to retrieve edges"})
		return
	}

	// Send the response
	c.JSON(http.StatusOK, gin.H{
		"nodes":       nodes,
		"connections": connections,
		"edges":       edges,
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

func DeleteDependency(ctx context.Context, driver neo4j.DriverWithContext, dep models.Dependency) error {
	query := `
    MATCH (source:Node {id: $localIp})-[r1:CONNECTS_TO]->(conn:Connection)-[r2:CONNECTS_TO]->(target:Node {id: $remoteIp})
    WHERE conn.protocol = $module AND conn.sourcePort = $localPort AND conn.targetPort = $remotePort
    DELETE r1, conn, r2
    `

	_, err := neo4j.ExecuteQuery(ctx, driver, query,
		map[string]interface{}{
			"localIp":    dep.LocalIp,
			"remoteIp":   dep.RemoteIp,
			"module":     dep.Module,
			"localPort":  dep.LocalPort,
			"remotePort": dep.RemotePort,
		},
		neo4j.EagerResultTransformer,
		neo4j.ExecuteQueryWithDatabase("neo4j"))

	return err
}

func HandleDependencyDeletions(c *gin.Context) {
	var dependencies []models.Dependency
	if err := c.ShouldBindJSON(&dependencies); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid input data"})
		return
	}

	ctx := context.Background()
	for _, dep := range dependencies {
		err := DeleteDependency(ctx, driver, dep)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": fmt.Sprintf("Failed to delete dependency: %v", err)})
			return
		}
	}

	c.JSON(http.StatusOK, gin.H{"message": "Dependencies deleted successfully"})
}
