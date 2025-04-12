package main

import (
	"fmt"
	"log"
	"os"

	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"

	"github.com/klrohias/lcsm-server/panel/auth"
)

var app *AppContext

func initDotenv() error {
	if err := godotenv.Load(); err != nil {
		return fmt.Errorf("error loading .env file: %v", err)
	}
	return nil
}

func initJwt() error {
	jwtSecret := os.Getenv("JWT_SECRET")
	if jwtSecret == "" {
		return fmt.Errorf("JWT_SECRET is not set")
	}

	app.JwtContext.SetJwtSecret(jwtSecret)

	return nil
}

func preloadAllEnvironment() {
	// Init AppContext
	var err error
	if app, err = PanelNewContext(); err != nil {
		log.Fatalf("Cannot new AppContext: %v", err)
	}

	// Init Jwt
	if err := initJwt(); err != nil {
		log.Fatalf("Jwt initialization failed: %v", err)
	}
}

func createRouters() *gin.Engine {
	r := gin.Default()

	// Initialize controllers
	userController := app.UserController
	runnerController := app.RunnerController
	systemController := app.SystemController

	// Initialize Middlewares
	jwtAuthMiddleware := auth.JwtAuthMiddleware(app.JwtContext)
	adminRoleMiddleware := auth.AdminRoleMiddleware()

	// Default routes
	r.GET("/Health", systemController.SystemHealth)

	// User routes
	userGroup := r.Group("/User")
	{
		userGroup.POST("/Authenticate", userController.Authenticate)
		userGroup.GET("", jwtAuthMiddleware, userController.CurrentUser)
	}

	// Runner routes (protected)
	runnerGroup := r.Group("/Runners", jwtAuthMiddleware, adminRoleMiddleware)
	{
		runnerGroup.GET("", runnerController.GetRunners)
		runnerGroup.PUT("", runnerController.CreateRunner)
		runnerGroup.POST("/:id", runnerController.UpdateRunner)
		runnerGroup.DELETE("/:id", runnerController.DeleteRunner)
	}

	return r
}

func startWebServer() {
	r := createRouters()

	// Launch server
	listenAddr := os.Getenv("LISTEN_ADDR")
	if listenAddr == "" {
		listenAddr = "8080"
	}

	if err := r.Run(listenAddr); err != nil {
		log.Fatalf("Server failed to start: %v", err)
	}
}

func main() {
	// Init environment
	if err := initDotenv(); err != nil {
		log.Printf("Warning: %v", err)
	}

	// Startup
	preloadAllEnvironment()
	startWebServer()
}
