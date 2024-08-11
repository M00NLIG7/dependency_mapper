package controllers

import (
	"depedency-mapper-server/initializers"
	"depedency-mapper-server/models"
	"net/http"

	"github.com/gin-gonic/gin"
)


type NodeRequest struct {
	SrcIP string    `json:"srcIP" binding:"required"`
	OS    models.OS `json:"os" binding:"required"`
}

func GetDependencies() []models.Dependency {
    var dependencies []models.Dependency
    initializers.DB.Find(&dependencies)
    return dependencies
}

func HandleDependency(c *gin.Context) {
    var dep *models.Dependency

	// Validate the input data
	if err := c.ShouldBindJSON(&dep); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid input data"})
		return
	}

     
    dep, err := models.AddDependency(initializers.DB, dep)
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add dependency"})
        return
    }

	// Add or get the local node
	localNode, err := models.AddNode(initializers.DB, dep.LocalIp, dep.LocalOS)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add or get local node"})
		return
	}

	// Add or get the remote node
	remoteNode, err := models.AddNode(initializers.DB, dep.RemoteIp, models.Unknown)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add or get remote node"})
		return
	}

	// Check if the edge (connection) already exists before creating it
	if _, err := models.AddEdge(initializers.DB, localNode.ID, remoteNode.SrcIP); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add edge"})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "Dependency added successfully"})
}

// Handler to create or update a node
func CreateOrUpdateNode(c *gin.Context) {
	var req NodeRequest

	// Bind the JSON payload to the struct
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// Interact with the database to create or update the node
	node, err := models.AddNode(initializers.DB, req.SrcIP, req.OS)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create or update node"})
		return
	}

	// Respond with the created or updated node
	c.JSON(http.StatusOK, gin.H{
		"id":    node.ID,
		"srcIP": node.SrcIP,
		"os":    node.OS,
	})
}


func GetNetworkGraph(c *gin.Context) {
	var nodes []models.Node
	var edges []models.Edge

	// Retrieve all nodes
	if err := initializers.DB.Find(&nodes).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to retrieve nodes"})
		return
	}

	// Retrieve all edges
	if err := initializers.DB.Preload("Source").Preload("Target").Find(&edges).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to retrieve edges"})
		return
	}

	// Construct the response
	nodeMap := make(map[uint]models.Node) // To track and include nodes only once
	for _, node := range nodes {
		nodeMap[node.ID] = node
	}

	// Prepare nodes in the expected format
	responseNodes := []map[string]interface{}{}
	for _, node := range nodeMap {
		responseNodes = append(responseNodes, map[string]interface{}{
			"id": node.SrcIP,
			"os": node.OS,
		})
	}

	// Prepare edges in the expected format
	responseEdges := []map[string]interface{}{}
	for _, edge := range edges {
		responseEdges = append(responseEdges, map[string]interface{}{
			"source": edge.Source.SrcIP,
			"target": edge.Target.SrcIP,
		})
	}

	// Send the response
	c.JSON(http.StatusOK, gin.H{
		"nodes": responseNodes,
		"edges": responseEdges,
	})
}
