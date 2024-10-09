
package api

import (
	"dependency-mapper/internal/controllers"
	"dependency-mapper/internal/middleware"

	"github.com/gin-gonic/gin"
)

// SetupRoutes configures all the routes for our application
func SetupRoutes(router *gin.Engine) {
	// API group
	api := router.Group("/api")
	{
		// Dependencies routes
		api.POST("/dependencies", controllers.HandleDependencies)
		api.GET("/dependencies", controllers.GetDependencies)

		// Graph data route
		api.GET("/graph-data", controllers.GetNetworkGraph)

		// Node routes
		api.GET("/node/:id", controllers.GetNode)
		api.POST("/node", controllers.CreateOrUpdateNode)
	}

	// Dashboard route (if still needed)
	router.GET("/node/:id", controllers.RenderNodeDashboard)

	// Map route (if still needed)
	router.GET("/map", controllers.RenderNetworkGraph)
}
