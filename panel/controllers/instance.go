package controllers

import (
	"strconv"

	"github.com/gofiber/fiber/v2"
	"github.com/klrohias/lcsm-server/common"
	"github.com/klrohias/lcsm-server/panel/db"
	"github.com/klrohias/lcsm-server/panel/services"
	"gorm.io/gorm"
)

type InstanceController struct {
	db            *gorm.DB
	runnerService *services.RunnerService
	logger        common.Logger
}

func NewInstanceController(
	db *db.DbContext,
	runnerService *services.RunnerService,
	logger common.Logger,
) *InstanceController {
	return &InstanceController{
		db:            db.DB,
		runnerService: runnerService,
		logger:        logger,
	}
}

func (ic *InstanceController) GetInstances(ctx *fiber.Ctx) error {
	idU64, err := strconv.ParseUint(ctx.Params("runnerId"), 10, 32)
	if err != nil {
		ic.logger.Debugf("Failed to parse runner ID: %v", err)
		return ctx.Status(fiber.StatusBadRequest).JSON(fiber.Map{"message": "Invalid runner ID"})
	}
	id := uint(idU64)

	// Open client
	client, err := ic.runnerService.GetClient(id)
	if err != nil {
		ic.logger.Debugf("Failed to get client: %v", err)
		return ctx.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"message": "Failed to connect runner"})
	}

	// Response
	instances, err := client.GetInstances()
	if err != nil {
		ic.logger.Debugf("Failed to get instances: %v", err)
		return ctx.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"message": "Failed to get instances"})
	}

	return ctx.Status(fiber.StatusOK).JSON(instances)
}
