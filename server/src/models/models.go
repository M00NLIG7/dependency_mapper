package models

import (
	"gorm.io/gorm"
)

// Maybe add CPE string for ndoe existing service comparison?

// Dependency represents a dependency between two nodes
type Dependency struct {
	ID          uint `gorm:"primaryKey"`
	NodeType    string
	Module      string
	LocalPort   int    `gorm:"uniqueIndex:idx_dep"`
	LocalIp     string `gorm:"uniqueIndex:idx_dep"`
	RemotePort  int    `gorm:"uniqueIndex:idx_dep"`
	RemoteIp    string `gorm:"uniqueIndex:idx_dep"`
	Description string
	Signature   string
}

type Node struct {
	ID    uint   `gorm:"primaryKey"`
	SrcIP string `gorm:"unique"`
}

type Edge struct {
	ID        uint   `gorm:"primaryKey"`
	SrcNodeID uint   `gorm:"index;uniqueIndex:idx_from_to"`
	DestIP    string `gorm:"uniqueIndex:idx_from_to"`
}

func AddNode(db *gorm.DB, srcIP string) (*Node, error) {
	var node Node
	if err := db.Where("src_ip = ?", srcIP).First(&node).Error; err == nil {
		return &node, nil
	}

	node = Node{SrcIP: srcIP}
	if err := db.Create(&node).Error; err != nil {
		return nil, err
	}
	return &node, nil
}

func AddEdge(db *gorm.DB, srcNodeID uint, destIP string) (*Edge, error) {
	var edge Edge
	if err := db.Where("src_node_id = ? AND dest_ip = ?", srcNodeID, destIP).First(&edge).Error; err == nil {
		return &edge, nil
	}

	edge = Edge{SrcNodeID: srcNodeID, DestIP: destIP}
	if err := db.Create(&edge).Error; err != nil {
		return nil, err
	}
	return &edge, nil
}

func GetNetworkGraph(db *gorm.DB) (map[string][]string, error) {
	var nodes []Node
	if err := db.Find(&nodes).Error; err != nil {
		return nil, err
	}

	graph := make(map[string][]string)
	for _, node := range nodes {
		graph[node.SrcIP] = []string{}
	}

	var edges []Edge
	if err := db.Find(&edges).Error; err != nil {
		return nil, err
	}

	for _, edge := range edges {
		var fromNode Node
		if err := db.First(&fromNode, edge.SrcNodeID).Error; err != nil {
			return nil, err
		}
		graph[fromNode.SrcIP] = append(graph[fromNode.SrcIP], edge.DestIP)
	}

	return graph, nil
}
