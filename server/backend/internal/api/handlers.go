package api

import (
    /*
	"dependency-mapper/internal/models"
	"net/http"
    */

	"github.com/gin-gonic/gin"
)

// HandleError is a utility function to handle errors in API responses
func HandleError(c *gin.Context, status int, message string) {
	c.JSON(status, gin.H{"error": message})
}

// RespondWithJSON is a utility function to respond with JSON data
func RespondWithJSON(c *gin.Context, status int, payload interface{}) {
	c.JSON(status, payload)
}
