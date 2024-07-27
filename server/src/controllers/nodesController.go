package controllers

import (
	"crypto/sha256"
	"depedency-mapper-server/initializers"
	"depedency-mapper-server/models"
	"encoding/hex"
	"net/http"

	"strconv"
	"github.com/gin-gonic/gin"
)

func generateNodeSignature(node models.Dependency) string {
	data := node.LocalIp + strconv.Itoa(node.LocalPort) + node.RemoteIp + strconv.Itoa(node.RemotePort) + node.Module + node.NodeType
	h := sha256.New()
	h.Write([]byte(data))
	hashBytes := h.Sum(nil)

	// Convert hash bytes to hex string
	hashString := hex.EncodeToString(hashBytes)
	return hashString
}

func HandleDependency(c *gin.Context) {
	var dep models.Dependency

    // Validate the input data
	if err := c.ShouldBindJSON(&dep); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid input data"})
		return
	}

	// Add the dependency to the database
	if err := initializers.DB.Create(&dep).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add dependency"})
		return
	}

	// Add or get the local node
	localNode, err := models.AddNode(initializers.DB, dep.LocalIp)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add node"})
		return
	}

	// Add edge between local node and remote IP
	_, err = models.AddEdge(initializers.DB, localNode.ID, dep.RemoteIp)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to add edge"})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "Node added successfully"})
}

func FetchGraph() (map[string][]string, error) {
    var nodes []models.Node
    if err := initializers.DB.Find(&nodes).Error; err != nil {
        return nil, err
    }

    graph := make(map[string][]string)
    for _, node := range nodes {
        graph[node.SrcIP] = []string{}
    }

    var edges []models.Edge
    if err := initializers.DB.Find(&edges).Error; err != nil {
        return nil, err
    }

    for _, edge := range edges {
        var fromNode models.Node
        if err := initializers.DB.First(&fromNode, edge.SrcNodeID).Error; err != nil {
            return nil, err
        }
        graph[fromNode.SrcIP] = append(graph[fromNode.SrcIP], edge.DestIP)
    }

    return graph, nil
}

