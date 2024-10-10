// File: internal/middleware/middleware.go

package middleware

import (
	"github.com/gin-gonic/gin"
	"net/http"
	"time"
	"log"
)

// AuthMiddleware checks for a valid authentication token
func AuthMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		token := c.GetHeader("Authorization")
		// Implement your token validation logic here
		if token == "" {
			c.JSON(http.StatusUnauthorized, gin.H{"error": "Authorization token required"})
			c.Abort()
			return
		}
		// If token is valid, call Next() to pass control to the next handler
		c.Next()
	}
}

// LoggingMiddleware logs the incoming HTTP request
func LoggingMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Start timer
		start := time.Now()

		// Process request
		c.Next()

		// Stop timer
		duration := time.Since(start)

		// Log request details
		log.Printf("%s %s %s %d %s",
			c.Request.Method,
			c.Request.URL.Path,
			c.ClientIP(),
			c.Writer.Status(),
			duration,
		)
	}
}

// ErrorHandlingMiddleware catches any panics and returns a 500 error
func ErrorHandlingMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		defer func() {
			if err := recover(); err != nil {
				log.Printf("Panic: %v", err)
				c.JSON(http.StatusInternalServerError, gin.H{"error": "Internal Server Error"})
			}
		}()
		c.Next()
	}
}

// CORSMiddleware handles Cross-Origin Resource Sharing (CORS)
func CORSMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Writer.Header().Set("Access-Control-Allow-Origin", "*")
		c.Writer.Header().Set("Access-Control-Allow-Credentials", "true")
		c.Writer.Header().Set("Access-Control-Allow-Headers", "Content-Type, Content-Length, Accept-Encoding, X-CSRF-Token, Authorization, accept, origin, Cache-Control, X-Requested-With")
		c.Writer.Header().Set("Access-Control-Allow-Methods", "POST, OPTIONS, GET, PUT, DELETE")

		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(204)
			return
		}

		c.Next()
	}
}

// RateLimitMiddleware implements a simple rate limiting mechanism
func RateLimitMiddleware(limit int, per time.Duration) gin.HandlerFunc {
	// This is a very basic implementation. For production, consider using a distributed rate limiter.
	type client struct {
		count    int
		lastSeen time.Time
	}
	clients := make(map[string]*client)

	return func(c *gin.Context) {
		ip := c.ClientIP()
		now := time.Now()
		if _, exists := clients[ip]; !exists {
			clients[ip] = &client{count: 0, lastSeen: now}
		}
		if now.Sub(clients[ip].lastSeen) > per {
			clients[ip].count = 0
			clients[ip].lastSeen = now
		}
		if clients[ip].count >= limit {
			c.JSON(http.StatusTooManyRequests, gin.H{"error": "Rate limit exceeded"})
			c.Abort()
			return
		}
		clients[ip].count++
		c.Next()
	}
}
