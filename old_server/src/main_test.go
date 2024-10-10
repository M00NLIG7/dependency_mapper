package main

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"

	"depedency-mapper-server/controllers"
	"depedency-mapper-server/initializers"
	"depedency-mapper-server/models"

	"github.com/gin-gonic/gin"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

// Setup and teardown for tests
func setupTestDB() (*gorm.DB, error) {
	db, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{
		Logger: logger.Default.LogMode(logger.Silent),
	})
	if err != nil {
		return nil, err
	}

	// Migrate the schema
	db.AutoMigrate(&models.Node{}, &models.Edge{}, &models.Dependency{})

	return db, nil
}

func teardownTestDB(db *gorm.DB) {
	db.Exec("DROP TABLE IF EXISTS nodes")
	db.Exec("DROP TABLE IF EXISTS edges")
	db.Exec("DROP TABLE IF EXISTS dependencies")
}

func TestHandleDependency(t *testing.T) {
	initializers.InitLogger() // Initialize the logger for the test

	db, err := setupTestDB()
	if err != nil {
		t.Fatalf("Failed to connect to test database: %v", err)
	}
	defer teardownTestDB(db)

	// Override the global DB instance with the test DB
	initializers.DB = db

	gin.SetMode(gin.TestMode)
	r := setupRouter()

	t.Run("Create Dependency", func(t *testing.T) {
		dep := models.Dependency{
			NodeType:    "exampleType",
			Module:      "exampleModule",
			LocalPort:   8080,
			LocalIp:     "192.168.1.1",
			RemotePort:  8081,
			RemoteIp:    "192.168.1.2",
			Description: "exampleDescription",
		}

		// Marshal dependency to JSON
		depJSON, _ := json.Marshal(dep)

		req, _ := http.NewRequest(http.MethodPost, "/handle-dependency", bytes.NewBuffer(depJSON))
		req.Header.Set("Content-Type", "application/json")

		// Perform the request
		w := httptest.NewRecorder()
		r.ServeHTTP(w, req)

        // Check if the response body is correct
        var response map[string]string
        err := json.Unmarshal(w.Body.Bytes(), &response)
        assert.NoError(t, err)

        // Check if the response message is correct
        assert.Equal(t, "Dependency added successfully", response["message"])

		// Check if the status code is 200 OK
		assert.Equal(t, http.StatusOK, w.Code)
	})
}

// setupRouter initializes the Gin router with routes from main.go
func setupRouter() *gin.Engine {
	r := gin.Default()

	r.POST("/handle-dependency", controllers.HandleDependency)

	return r
}

