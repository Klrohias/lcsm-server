package main

import (
	"log"
	"os"

	"github.com/gofiber/fiber/v2"
	"go.uber.org/dig"

	"github.com/klrohias/lcsm-server/common"
	"github.com/klrohias/lcsm-server/panel/controllers"
	"github.com/klrohias/lcsm-server/panel/db"
	"github.com/klrohias/lcsm-server/panel/services"
)

type appContext struct {
	dbContext *db.DbContext

	userController     *controllers.UserController
	runnerController   *controllers.RunnerController
	systemController   *controllers.SystemController
	instanceController *controllers.InstanceController
	runnerService      *services.RunnerService
	authService        *services.AuthService
	logger             common.Logger
}

func newAppContext(
	dbContext *db.DbContext,
	userController *controllers.UserController,
	runnerController *controllers.RunnerController,
	systemController *controllers.SystemController,
	instanceController *controllers.InstanceController,
	runnerService *services.RunnerService,
	authService *services.AuthService,
	logger common.Logger,
) *appContext {
	return &appContext{
		dbContext,
		userController,
		runnerController,
		systemController,
		instanceController,
		runnerService,
		authService,
		logger,
	}
}

func setupAppContext(appContext *appContext) {
	authService := appContext.authService
	if defaultJwtSecret, defaultJwtSecretExists := os.LookupEnv("LCSM_JWT_SECRET"); defaultJwtSecretExists {
		authService.SetJwtSecret(defaultJwtSecret)
	} else {
		appContext.logger.Warnf("Environment variable LCSM_JWT_SECRET is not set")
		authService.SetJwtSecret("")
	}
}

func newWebServer(appContext *appContext) *fiber.App {
	app := fiber.New()

	// Controllers
	userController := appContext.userController
	runnerController := appContext.runnerController
	systemController := appContext.systemController
	instanceController := appContext.instanceController

	// Initialize Middlewares
	jwtAuthMiddleware := appContext.authService.JwtAuthMiddleware()
	adminRoleMiddleware := appContext.authService.AdminRoleMiddleware()

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

func getListenAddr() string {
	listenAddr := ":8008"
	if listenAddrFromEnv, exists := os.LookupEnv("LCSM_LISTEN_ADDR"); exists {
		listenAddr = listenAddrFromEnv
	}
	return listenAddr
}

func makeContext() *dig.Container {
	c := dig.New()

	c.Provide(newAppContext)

	// Controller
	c.Provide(controllers.NewInstanceController)
	c.Provide(controllers.NewRunnerController)
	c.Provide(controllers.NewSystemController)
	c.Provide(controllers.NewUserController)

	// Services
	c.Provide(services.NewRunnerService)
	c.Provide(services.NewAuthService)

	// Misc
	c.Provide(db.NewDbContext)
	c.Provide(common.NewDefaultLogger, dig.As(new(common.Logger)))

	return c
}

func main() {
	c := makeContext()
	c.Invoke(func(appContext *appContext) {
		setupAppContext(appContext)

		s := newWebServer(appContext)

		if err := s.Listen(getListenAddr()); err != nil {
			log.Fatalf("Server failed to start: %v", err)
		}
	})
}
