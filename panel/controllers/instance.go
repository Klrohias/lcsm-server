package controllers

import (
	"errors"
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

type getInstancesBody struct {
	Page     uint `json:"page"`
	PageSize uint `json:"pageSize"`
}

func (ic *InstanceController) GetInstances(ctx *fiber.Ctx) error {
	idU64, err := strconv.ParseUint(ctx.Params("runnerId"), 10, 32)
	if err != nil {
		ic.logger.Debugf("Failed to parse runner ID: %v", err)
		return ctx.Status(fiber.StatusBadRequest).JSON(fiber.Map{"message": "Invalid runner ID"})
	}
	id := uint(idU64)

	body := &getInstancesBody{}
	if err = ctx.BodyParser(body); err != nil && !errors.Is(err, fiber.ErrUnprocessableEntity) {
		ic.logger.Debugf("Failed to parse getInstancesBody: %v", err)
		return ctx.Status(fiber.StatusBadRequest).JSON(fiber.Map{"message": "Invalid body"})
	}

	// Open client
	client, err := ic.runnerService.GetClient(uint(id))
	if err != nil {
		ic.logger.Debugf("Failed to get client: %v", err)
		return ctx.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"message": "Failed to connect runner"})
	}

	// Response
	instances, err := client.GetInstances(int(body.Page), int(body.PageSize))
	if err != nil {
		ic.logger.Debugf("Failed to get instances: %v", err)
		return ctx.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"message": "Failed to get instances"})
	}

	return ctx.Status(fiber.StatusOK).JSON(instances)
}
