package initializers

import (
	"go.uber.org/zap"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

var DB *gorm.DB

func ConnectToDb() {
	var err error
	DB, err = gorm.Open(sqlite.Open("depmap.db"), &gorm.Config{})

	if DB == nil {
		zap.S().Infof("Unspecifed error opening db")
		panic("failed to connect database")
	}

	if err != nil {
		zap.S().Infof("Error opening db: %s", err)
		panic("failed to connect database")
	}

}
