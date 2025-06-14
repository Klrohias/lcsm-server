package main

import (
	"log"
	"os"

	"github.com/gofiber/contrib/websocket"
	"github.com/gofiber/fiber/v2"
	"github.com/klrohias/lcsm-server/common"
	"github.com/klrohias/lcsm-server/runner/db"
	"github.com/klrohias/lcsm-server/runner/services"
	"go.uber.org/dig"
)

type appContext struct {
	logger               common.Logger
	controlSocketService *services.ControlSocketService
}

func newAppContext(
	logger common.Logger,
	controlSocketService *services.ControlSocketService,
) *appContext {
	return &appContext{
		logger,
		controlSocketService,
	}
}

func newWebServer(appContext *appContext) *fiber.App {
	app := fiber.New()

	app.Get("/lcsm-node.socket", websocket.New(func(c *websocket.Conn) {
		appContext.controlSocketService.HandleWebSocket(c)
	}))

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

	// Services
	c.Provide(services.NewProcessManagementService)
	c.Provide(services.NewControlSocketService)

	// Misc
	c.Provide(db.NewDbContext)
	c.Provide(common.NewDefaultLogger, dig.As(new(common.Logger)))

	return c
}

func setupContext(appContext *appContext) {
	authToken := os.Getenv("LCSM_AUTH_TOKEN")
	if authToken == "" {
		log.Fatal("LCSM_AUTH_TOKEN environment variable not set")
	}
	appContext.controlSocketService.SetAuthToken(authToken)
}

func main() {
	c := makeContext()
	c.Invoke(func(appContext *appContext) {
		setupContext(appContext)

		s := newWebServer(appContext)
		if err := s.Listen(getListenAddr()); err != nil {
			log.Fatalf("Server failed to start: %v", err)
		}
	})
}
