package main

import (
	"depedency-mapper-server/controllers"
	"depedency-mapper-server/initializers"
	"encoding/json"
	"net/http"
    "fmt"

	"github.com/gin-gonic/gin"
)

func init() {
	initializers.InitLogger()
	initializers.ConnectToDb()
	initializers.SyncDatabase()
}

func main() {
	router := gin.Default()
	//router.Static("/dist", "./dist")
	router.LoadHTMLGlob("templates/*/*.html")

	router.GET("/ping", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"message": "pong",
		})
	})

	router.GET("/", func(c *gin.Context) {
		c.HTML(http.StatusOK, "map", gin.H{})
	})

	router.POST("/addnode", controllers.HandleDependency)

	router.GET("/map", func(c *gin.Context) {
		networkGraph, err := controllers.FetchGraph()

        if err != nil {
            fmt.Println(err)
        }

		// Convert graph to JSON Convert map to json string
		jsonStr, err := json.Marshal(networkGraph)
		if err != nil {
			fmt.Println(err)
		}

		c.HTML(http.StatusOK, "map", gin.H{"graph": string(jsonStr)})
	})

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
