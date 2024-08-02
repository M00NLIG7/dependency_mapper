package models

import (
	"gorm.io/gorm"
    "encoding/json"
    "database/sql/driver"

    "fmt"
    "errors"
)

type OS string

const (
	Linux   OS = "Linux"
	Windows OS = "Windows"
	Mac     OS = "Mac"
	Unknown OS = "Unknown"
)

// Implement the UnmarshalJSON method for OS
func (os *OS) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return err
	}

	switch s {
	case string(Linux), string(Windows), string(Mac):
		*os = OS(s)
	default:
		*os = Unknown // or return an error if you want to reject unknown OS types
	}
	return nil
}

// Implement the Scan and Value methods for database interaction
func (os *OS) Scan(value interface{}) error {
	if v, ok := value.(string); ok {
		*os = OS(v)
		return nil
	}
	return errors.New("failed to scan OS")
}

func (os OS) Value() (driver.Value, error) {
	return string(os), nil
}

// Maybe add CPE string for ndoe existing service comparison?

// Dependency represents a dependency between two nodes
type Dependency struct {
	ID          uint `gorm:"primaryKey"`
	NodeType    string
	Module      string
	LocalPort   int    `gorm:"uniqueIndex:idx_dep"`
	LocalIp     string `gorm:"uniqueIndex:idx_dep"`
    LocalOS     OS
	RemotePort  int    `gorm:"uniqueIndex:idx_dep"`
	RemoteIp    string `gorm:"uniqueIndex:idx_dep"`
	Description string
}

type Node struct {
	ID    uint   `gorm:"primaryKey"`
	SrcIP string `gorm:"unique"` // Ensure unique constraint on src_ip
	OS    OS `gorm:"type:varchar(10)"`
}

// Edge represents a connection between two nodes
type Edge struct {
	ID        uint   `gorm:"primaryKey"`
	SourceID  uint   `gorm:"index"`
	Source    Node   `gorm:"foreignKey:SourceID"`
	TargetID  uint   `gorm:"index"`
	Target    Node   `gorm:"foreignKey:TargetID"`
}

func AddNode(db *gorm.DB, srcIP string, os OS) (*Node, error) {
	var node Node

	// Check if the node already exists
	if err := db.Where("src_ip = ?", srcIP).First(&node).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			// Node doesn't exist, create a new one
			node = Node{SrcIP: srcIP, OS: os}
			if err := db.Create(&node).Error; err != nil {
				return nil, fmt.Errorf("failed to create node: %w", err)
			}
		} else {
			// Another error occurred
			return nil, fmt.Errorf("failed to query node: %w", err)
		}
	} else {
		// Node exists, update it if necessary
		if node.OS != os {
			node.OS = os
			if err := db.Save(&node).Error; err != nil {
				return nil, fmt.Errorf("failed to update node: %w", err)
			}
		}
	}

	// Return the existing or newly created node
	return &node, nil
}

func AddEdge(db *gorm.DB, sourceID uint, targetIP string) (*Edge, error) {
	var targetNode Node
	if err := db.Where("src_ip = ?", targetIP).First(&targetNode).Error; err != nil {
		return nil, fmt.Errorf("failed to find target node: %w", err)
	}

	// Check if the edge already exists
	var edge Edge
	if err := db.Where("source_id = ? AND target_id = ?", sourceID, targetNode.ID).First(&edge).Error; err == nil {
		// Edge already exists, return it
		return &edge, nil
	} else if !errors.Is(err, gorm.ErrRecordNotFound) {
		// Another error occurred
		return nil, fmt.Errorf("failed to check existing edge: %w", err)
	}

	// Edge doesn't exist, create it
	edge = Edge{SourceID: sourceID, TargetID: targetNode.ID}
	if err := db.Create(&edge).Error; err != nil {
		return nil, fmt.Errorf("failed to create edge: %w", err)
	}

	return &edge, nil
}

