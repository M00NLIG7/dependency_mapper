package controllers

import (
	"depedency-mapper-server/initializers"
	"depedency-mapper-server/models"
	"fmt"
	"net/http"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
)

func edgeCheck() {
	fmt.Println("Yeah we checkin edges bruh")
}

func AddNode(c *gin.Context) {
	var node models.Node
	if err := c.BindJSON(&node); err != nil {
		zap.S().Info("Error binding JSON data")
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	var allNodes []models.Node
	nodesResult := initializers.DB.Find(&allNodes)

	if nodesResult.Error != nil {
		zap.S().Error("Error: ", nodesResult.Error)
		c.JSON(http.StatusBadRequest, gin.H{"error": nodesResult.Error})
		return
	}

	for _, savedNode := range allNodes {
		// Save DB node to internal node type
		transformedNode := models.Node{
			NodeType:    savedNode.NodeType,
			Module:      savedNode.Module,
			DestPort:    savedNode.DestPort,
			DestIp:      savedNode.DestIp,
			SrcPort:     savedNode.SrcPort,
			SrcIp:       savedNode.SrcIp,
			Description: savedNode.Description,
		}
		if transformedNode == node {
			c.JSON(http.StatusBadRequest, gin.H{"error": "Node already exists!"})
			return
		}
	}

	result := initializers.DB.Create(&node)

	if result.Error != nil {
		zap.S().Info("Error adding new data to DB")
		c.JSON(http.StatusBadRequest, gin.H{"error": result.Error})
		return
	}

	// List some debugging stats
	zap.S().Infof("Rows affected: %v", result.RowsAffected)
	c.JSON(http.StatusOK, gin.H{"message": "DB updated!"})
}

// Test data

// node := models.Node{
// 	NodeType:    "service",
// 	Module:      "core",
// 	DestPort:    3306,
// 	DestIp:      "192.168.60.33",
// 	SrcPort:     60888,
// 	SrcIp:       "192.168.60.22",
// 	Description: "This is test data!",
// }
