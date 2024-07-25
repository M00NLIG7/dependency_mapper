package models

// need to add constraints

// Maybe add CPE string for ndoe existing service comparison?

// Database model
type Node struct {
	ID          uint `gorm:"primaryKey"`
	NodeType    string
	Module      string
	LocalPort   int
	LocalIp     string
	RemotePort  int
	RemoteIp    string
	Description string
	Signature   string
}

// Database model
type Edge struct {
	ID           uint `gorm:"primaryKey"`
	SrcNodeID    uint // Foreign key for the source Node
	DestNodeID   uint // Foreign key for the destination Node
	Relationship string
	SrcNode      Node `gorm:"foreignKey:SrcNodeID"`  // Specifies SrcNode as the foreign key relationship
	DestNode     Node `gorm:"foreignKey:DestNodeID"` // Specifies DestNode as the foreign key relationship
}
