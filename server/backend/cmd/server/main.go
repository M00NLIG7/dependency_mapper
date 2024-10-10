package main

import (
	"log"
	"net/http"
	"net/http/httputil"
	"net/url"
	"os"

	"dependency-mapper/internal/controllers"
	"dependency-mapper/internal/initializers"
	"dependency-mapper/internal/models"
	"github.com/gin-gonic/gin"
)

type ServerData struct {
	ServerName     string
	IPAddress      string
	ConnectionType string
	Port           string
	Status         string
}

func init() {
	// Initialize logger
	initializers.InitLogger()
	// Connect to SQL database (if still needed)
	initializers.ConnectToDb()
	// Sync SQL database (if still needed)
	initializers.SyncDatabase()
	// Initialize Neo4j driver
	err := controllers.InitDriver("bolt://localhost:7687", "neo4j", "secretgraph")
	if err != nil {
		log.Fatalf("Failed to initialize Neo4j driver: %v", err)
	}
}

func main() {
	router := gin.Default()

	// Serve static files
	router.Static("/static", "./static")

	// API routes (available in both production and development)
	api := router.Group("/api")
	{
		api.POST("/dependencies", controllers.HandleDependencies)
		api.GET("/graph-data", controllers.GetNetworkGraph)
		api.GET("/update-dependencies", func(c *gin.Context) {
			dependencies, err := controllers.GetDependencies()
			if err != nil {
				log.Printf("Failed to get dependencies: %v", err)
				dependencies = []models.Dependency{}
			}
			c.JSON(http.StatusOK, dependencies)
		})

        /*
		api.GET("/node/:id", func(c *gin.Context) {
			nodeData, err := controllers.GetNodeData(c.Param("id"))
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, nodeData)
		})

        */
		api.GET("/test", func(c *gin.Context) {
			data := map[string]ServerData{
				"Server1": {
					ServerName:     "Database Server",
					IPAddress:      "192.168.1.101",
					ConnectionType: "MySQL",
					Port:           "3306",
					Status:         "Connected",
				},
				"Server2": {
					ServerName:     "Cache Server",
					IPAddress:      "192.168.1.102",
					ConnectionType: "Redis",
					Port:           "6379",
					Status:         "Disconnected",
				},
				"Server3": {
					ServerName:     "Logging Server",
					IPAddress:      "192.168.1.103",
					ConnectionType: "Elasticsearch",
					Port:           "9200",
					Status:         "Connected",
				},
			}
			c.JSON(http.StatusOK, data)
		})
		// Add any other API routes here
	}

	if gin.Mode() == gin.ReleaseMode {
		// Serve the Next.js app in production
		router.NoRoute(func(c *gin.Context) {
			path := c.Request.URL.Path
			if path == "/" {
				c.File("./frontend/out/index.html")
			} else if fileExists("./frontend/out" + path) {
				c.File("./frontend/out" + path)
			} else {
				c.File("./frontend/out/index.html") // Fallback for client-side routing
			}
		})
	} else {
		// In development, proxy requests to the Next.js dev server
		router.NoRoute(func(c *gin.Context) {
			targetURL := "http://localhost:3000" + c.Request.URL.Path
			if c.Request.URL.RawQuery != "" {
				targetURL += "?" + c.Request.URL.RawQuery
			}
			proxy := &httputil.ReverseProxy{
				Director: func(req *http.Request) {
					req.URL, _ = url.Parse(targetURL)
					req.Header.Set("X-Forwarded-Host", req.Host)
					req.Host = "localhost:3000"
				},
			}
			proxy.ServeHTTP(c.Writer, c.Request)
		})
	}

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	log.Printf("Server is running on http://localhost:%s", port)
	router.Run(":" + port)
}

// Helper function to check if a file exists
func fileExists(filename string) bool {
	_, err := os.Stat(filename)
	return err == nil
}
