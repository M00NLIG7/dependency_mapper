package controllers

import (
	"crypto/sha256"
	"depedency-mapper-server/initializers"
	"depedency-mapper-server/models"
	"encoding/hex"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
)

var AdjacencyMatrix map[string][]string

func EdgeCheck() {
	var allNodes []models.Node
	uniqueNodesResult := initializers.DB.Find(&allNodes)
	zap.S().Infof("There are %v records: %v", uniqueNodesResult.RowsAffected, allNodes)

	if uniqueNodesResult.Error != nil {
		zap.S().Error("Error: ", uniqueNodesResult.Error)
		return
	}

	for _, node := range allNodes {
		if AdjacencyMatrix == nil {
			// intialize map
			zap.S().Info("Initializing map...")
			AdjacencyMatrix = make(map[string][]string)
		}

		// check if key exists
		_, ok := AdjacencyMatrix[node.LocalIp]
		if !ok {
			zap.S().Infof("%s does not exist", node.LocalIp)
			AdjacencyMatrix[node.LocalIp] = []string{}
		}
		// Check if the destination IP already exists in the specifed key value
		counter := 0
		if node.RemoteIp != "0.0.0.0" {
			for _, RemoteIp := range AdjacencyMatrix[node.LocalIp] {
				if RemoteIp == node.RemoteIp {
					zap.S().Infof("%s destination ID already loaded", RemoteIp)
					counter++
				}
			}
		} else {
			counter++
		}

		if counter == 0 {
			// Add dest ip to EdgeMap slice
			AdjacencyMatrix[node.LocalIp] = append(AdjacencyMatrix[node.LocalIp], node.RemoteIp)
		}
	}
}

func EdgeCheckEndpoint(c *gin.Context) {
	EdgeCheck()
	zap.S().Infof("Current edge structure: %v", AdjacencyMatrix)
}

func generateNodeSignature(node models.Node) string {
	data := node.LocalIp + strconv.Itoa(node.LocalPort) + node.RemoteIp + strconv.Itoa(node.RemotePort) + node.Module + node.NodeType
	h := sha256.New()
	h.Write([]byte(data))
	hashBytes := h.Sum(nil)

	// Convert hash bytes to hex string
	hashString := hex.EncodeToString(hashBytes)
	return hashString
}

func AddNode(c *gin.Context) {
	var newNode models.Node
	if err := c.BindJSON(&newNode); err != nil {
		zap.S().Info("Error binding JSON data")
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// Create signature for new node
	newNodeSig := generateNodeSignature(newNode)
	// Assign new signature to the new node struct
	newNode.Signature = newNodeSig

	var allNodes []models.Node
	nodesResult := initializers.DB.Find(&allNodes)

	if nodesResult.Error != nil {
		zap.S().Error("Error: ", nodesResult.Error)
		c.JSON(http.StatusBadRequest, gin.H{"error": nodesResult.Error})
		return
	}

	for _, dbNode := range allNodes {
		if dbNode.Signature == newNodeSig {
			zap.S().Error("Node already exists!")
			c.JSON(http.StatusBadRequest, gin.H{"error": "Node already exists"})
			return
		}
	}

	result := initializers.DB.Create(&newNode)

	if result.Error != nil {
		zap.S().Info("Error adding new data to DB")
		c.JSON(http.StatusBadRequest, gin.H{"error": result.Error})
		return
	}

	EdgeCheck()

	// List some debugging stats
	zap.S().Infof("Rows affected: %v", result.RowsAffected)
	zap.S().Infof("Current edge structure: %v", AdjacencyMatrix)
	c.JSON(http.StatusOK, gin.H{"message": "DB updated!"})
}
