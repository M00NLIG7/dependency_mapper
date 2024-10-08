package main

import (
	"depedency-mapper-server/components"
	"depedency-mapper-server/controllers"
	"depedency-mapper-server/initializers"
	"depedency-mapper-server/models"
	"log"
	"net/http"

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
    router.Static("/static", "./static")

    router.GET("/", func(c *gin.Context) {
        c.HTML(http.StatusOK, "map", gin.H{})
    })

    /*
    router.POST("/addnode", controllers.HandleDependency)
    router.POST("/api/dependency", controllers.HandleDependency)
    */
    router.POST("/api/dependencies", controllers.HandleDependencies)

    router.GET("/node/:id", func(c *gin.Context) {
        components.NodeDashboard().Render(c.Request.Context(), c.Writer)
    })

    router.GET("/map", func(c *gin.Context) {
        components.NetworkGraph().Render(c.Request.Context(), c.Writer)
    })

    router.GET("/update-dependencies", func(c *gin.Context) {
        dependencies, err := controllers.GetDependencies()
        if err != nil {
            log.Printf("Failed to get dependencies: %v", err)
            dependencies = []models.Dependency{};
        }

        components.Dependencies(dependencies).Render(c.Request.Context(), c.Writer)
    })

    router.GET("/test", func(c *gin.Context) {
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
        c.HTML(http.StatusOK, "main", data)
    })

    router.GET("/api/graph-data", controllers.GetNetworkGraph)
    /*
    router.POST("/api/node", controllers.CreateOrUpdateNode)
    */

    router.Run()
}
