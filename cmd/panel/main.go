package main

import (
	"fmt"
	"log"
	"os"

	"github.com/gofiber/fiber/v2"
	"github.com/joho/godotenv"

	"github.com/klrohias/lcsm-server/panel/auth"
)

var appContext *AppContext

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

	appContext.JwtContext.SetJwtSecret(jwtSecret)

	return nil
}

func prepareContext() {
	// Init environment
	if err := initDotenv(); err != nil {
		log.Printf("Warning: %v", err)
	}

	// Init AppContext
	var err error
	if appContext, err = BuildAppContext(); err != nil {
		log.Fatalf("Cannot new AppContext: %v", err)
	}

	// Init Jwt
	if err := initJwt(); err != nil {
		log.Fatalf("Jwt initialization failed: %v", err)
	}
}

func createRouters() *fiber.App {
	app := fiber.New()

	// Initialize controllers
	userController := appContext.UserController
	runnerController := appContext.RunnerController
	systemController := appContext.SystemController
	instanceController := appContext.InstanceController

	// Initialize Middlewares
	jwtAuthMiddleware := auth.JwtAuthMiddleware(appContext.JwtContext)
	adminRoleMiddleware := auth.AdminRoleMiddleware()

	// Default routes
	app.Get("/Health", systemController.SystemHealth)

	// User routes
	userGroup := app.Group("/User")
	{
		userGroup.Post("/Authenticate", userController.Authenticate)
		userGroup.Get("", jwtAuthMiddleware, userController.CurrentUser)
	}

	// Runner routes (protected)
	runnerGroup := app.Group("/Runners", jwtAuthMiddleware, adminRoleMiddleware)
	{
		runnerGroup.Get("", runnerController.GetRunners)
		runnerGroup.Put("", runnerController.CreateRunner)
		runnerGroup.Post("/:id", runnerController.UpdateRunner)
		runnerGroup.Delete("/:id", runnerController.DeleteRunner)
	}

	// Instance routes
	// TODO
	instanceGroup := app.Group("/Instances/:runnerId", jwtAuthMiddleware)
	{
		instanceGroup.Get("", instanceController.GetInstances)
	}

	return app
}

func startWebServer() {
	app := createRouters()

	// Launch server
	listenAddr := os.Getenv("LISTEN_ADDR")
	if listenAddr == "" {
		listenAddr = ":8080"
	}

	if err := app.Listen(listenAddr); err != nil {
		log.Fatalf("Server failed to start: %v", err)
	}
}

func main() {
	prepareContext()
	startWebServer()
}
