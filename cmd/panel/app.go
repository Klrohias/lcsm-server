//go:build wireinject
// +build wireinject

package main

import (
	"github.com/google/wire"
	"github.com/klrohias/lcsm-server/panel/auth"
	"github.com/klrohias/lcsm-server/panel/controllers"
	"github.com/klrohias/lcsm-server/panel/db"
)

var ControllerSet = wire.NewSet(controllers.NewUserController,
	controllers.NewRunnerController,
	controllers.NewSystemController)

type AppContext struct {
	DbContext        *db.DbContext
	JwtContext       *auth.JwtContext
	UserController   *controllers.UserController
	RunnerController *controllers.RunnerController
	SystemController *controllers.SystemController
}

func NewAppContext(
	dbContext *db.DbContext,
	jwtContext *auth.JwtContext,
	userController *controllers.UserController,
	runnerController *controllers.RunnerController,
	systemController *controllers.SystemController) *AppContext {
	return &AppContext{
		DbContext:        dbContext,
		JwtContext:       jwtContext,
		UserController:   userController,
		RunnerController: runnerController,
		SystemController: systemController,
	}
}

func PanelNewContext() (*AppContext, error) {
	wire.Build(NewAppContext, db.NewDbContext, auth.NewJwtContext, ControllerSet)
	return &AppContext{}, nil
}
