//go:build wireinject
// +build wireinject

package main

import (
	"github.com/google/wire"
	"github.com/klrohias/lcsm-server/common"
	"github.com/klrohias/lcsm-server/panel/auth"
	"github.com/klrohias/lcsm-server/panel/controllers"
	"github.com/klrohias/lcsm-server/panel/db"
	"github.com/klrohias/lcsm-server/panel/services"
)

var ControllerSet = wire.NewSet(controllers.NewUserController,
	controllers.NewRunnerController,
	controllers.NewSystemController,
	controllers.NewInstanceController)

var ServiceSet = wire.NewSet(
	services.NewRunnerService)

type AppContext struct {
	DbContext          *db.DbContext
	JwtContext         *auth.JwtContext
	UserController     *controllers.UserController
	RunnerController   *controllers.RunnerController
	SystemController   *controllers.SystemController
	InstanceController *controllers.InstanceController
	RunnerService      *services.RunnerService
	Logger             common.Logger
}

func NewAppContext(
	dbContext *db.DbContext,
	jwtContext *auth.JwtContext,
	userController *controllers.UserController,
	runnerController *controllers.RunnerController,
	systemController *controllers.SystemController,
	instanceController *controllers.InstanceController,
	runnerService *services.RunnerService,
	logger common.Logger,
) *AppContext {
	return &AppContext{
		DbContext:          dbContext,
		JwtContext:         jwtContext,
		UserController:     userController,
		RunnerController:   runnerController,
		SystemController:   systemController,
		InstanceController: instanceController,
		RunnerService:      runnerService,
		Logger:             logger,
	}
}

func BuildAppContext() (*AppContext, error) {
	wire.Build(NewAppContext,
		db.NewDbContext,
		auth.NewJwtContext,
		common.NewDefaultLogger,
		ControllerSet,
		ServiceSet,
		wire.Bind(new(common.Logger), new(*common.DefaultLogger)))

	return &AppContext{}, nil
}
