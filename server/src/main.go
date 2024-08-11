package main

import (
	"depedency-mapper-server/controllers"
	"depedency-mapper-server/initializers"
    "depedency-mapper-server/components"
	"net/http"

    //    "depedency-mapper-server/templates"

	"github.com/gin-gonic/gin"
)

type Node struct {
    ip string
}

func init() {
	initializers.InitLogger()
	initializers.ConnectToDb()
	initializers.SyncDatabase()
}

type ServerData struct {
    ServerName     string
    IPAddress      string
    ConnectionType string
    Port           string
    Status         string
}


func main() {
	router := gin.Default()

    router.Static("/static", "./static")

	//router.Static("/dist", "./dist")
	router.LoadHTMLGlob("templates/*/*.html")

	router.GET("/", func(c *gin.Context) {
		c.HTML(http.StatusOK, "map", gin.H{})
	})

	router.POST("/addnode", controllers.HandleDependency)

    router.POST("/api/dependency", controllers.HandleDependency)

    // Define a route for the index page
    router.GET("/node/:id", func(c *gin.Context) {
        components.NodeDashboard().Render(c.Request.Context(), c.Writer)
	})

	router.GET("/map", func(c *gin.Context) {
		c.HTML(http.StatusOK, "map", nil)
	})

    router.GET("/update-dependencies", func(c *gin.Context) {
        dependencies := controllers.GetDependencies()
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

    router.POST("/api/node", controllers.CreateOrUpdateNode)

	router.Run()
}
