package main

import (
	"dependency-mapper/internal/api"
	"dependency-mapper/internal/controllers"
	"dependency-mapper/internal/initializers"
	"log"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
)

func init() {
	initializers.InitLogger()
	initializers.ConnectToDb()
	initializers.SyncDatabase()
	err := controllers.InitDriver("bolt://localhost:7687", "neo4j", "secretgraph")
	if err != nil {
		log.Fatalf("Failed to initialize Neo4j driver: %v", err)
	}
}

func main() {
	router := gin.Default()

	// Configure CORS
	config := cors.DefaultConfig()
	config.AllowOrigins = []string{"http://localhost:3000"} // Add your frontend URL
	router.Use(cors.New(config))

	// Serve static files
	router.Static("/static", "./static")

	// Set up API routes
	api.SetupRoutes(router)

	router.Run()
}

