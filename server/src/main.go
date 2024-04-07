package main

import (
	"depedency-mapper-server/controllers"
	"depedency-mapper-server/initializers"
	"net/http"

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

	router.POST("/addnode", controllers.AddNode)

	router.Run()
}
