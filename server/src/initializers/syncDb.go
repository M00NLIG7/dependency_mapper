package initializers

import (
	"depedency-mapper-server/models"
)

func SyncDatabase() {
	// Code to sync the database
	DB.AutoMigrate(&models.Node{}, &models.Edge{})
}
