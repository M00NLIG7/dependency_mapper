package main

import (
	"depedency-mapper-server/controllers"
	"depedency-mapper-server/initializers"
	"net/http"

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

func main() {
	router := gin.Default()
	//router.Static("/dist", "./dist")
	router.LoadHTMLGlob("templates/*/*.html")

	router.GET("/", func(c *gin.Context) {
		c.HTML(http.StatusOK, "map", gin.H{})
	})

	router.POST("/addnode", controllers.HandleDependency)

    router.POST("/api/dependency", controllers.HandleDependency)

	router.GET("/map", func(c *gin.Context) {
		c.HTML(http.StatusOK, "map", nil)
	})


    router.GET("/api/graph-data", controllers.GetNetworkGraph)

    router.POST("/api/node", controllers.CreateOrUpdateNode)


	// SSE endpoint: Maybe TODO?
	/*
		router.GET("/events", func(c *gin.Context) {
			c.Stream(func(w io.Writer) bool {
				values := controllers.FetchGraph()

				c.SSEvent("update", values)
				time.Sleep(10 * time.Second) // Adjust the interval as needed
				return true
			})
		})
	*/

	router.Run()
}
