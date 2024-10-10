package initializers

import (
	"dependency-mapper/internal/models"
)

func SyncDatabase() {
	// Code to sync the database
	DB.AutoMigrate(&models.Node{}, &models.Edge{}, &models.Dependency{})
}
