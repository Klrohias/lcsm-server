package controllers

import (
	"errors"
	"strconv"

	"github.com/gofiber/fiber/v2"
	"github.com/klrohias/lcsm-server/common"
	"github.com/klrohias/lcsm-server/panel"
	"github.com/klrohias/lcsm-server/panel/services"
	"gorm.io/gorm"
)

type InstanceController struct {
	db            *gorm.DB
	runnerService *services.RunnerService
	logger        common.Logger
}

func NewInstanceController(
	db *gorm.DB,
	runnerService *services.RunnerService,
	logger common.Logger,
) *InstanceController {
	return &InstanceController{
		db,
		runnerService,
		logger,
	}
}

type getInstancesBody struct {
	Page     uint `json:"page"`
	PageSize uint `json:"pageSize"`
}

func (ic *InstanceController) GetInstances(ctx *fiber.Ctx) error {
	// Parse runner ID
	idU64, err := strconv.ParseUint(ctx.Params("runnerId"), 10, 32)
	if err != nil {
		ic.logger.Debugf("Failed to parse runner ID: %v", err)
		return ctx.Status(fiber.StatusBadRequest).JSON(panel.ErrorInvalidRunnerID)
	}
	id := uint(idU64)

	// Parse body
	body := &getInstancesBody{}
	if err = ctx.BodyParser(body); err != nil && !errors.Is(err, fiber.ErrUnprocessableEntity) {
		ic.logger.Debugf("Failed to parse getInstancesBody: %v", err)
		return ctx.Status(fiber.StatusBadRequest).JSON(panel.ErrorInvalidBody)
	}

	// Open client
	client, err := ic.runnerService.GetClient(uint(id))
	if err != nil {
		ic.logger.Debugf("Failed to get client: %v", err)
		return ctx.Status(fiber.StatusInternalServerError).JSON(panel.ErrorInternal)
	}

	// Response
	instances, err := client.GetInstances(int(body.Page), int(body.PageSize))
	if err != nil {
		ic.logger.Debugf("Failed to get instances: %v", err)
		return ctx.Status(fiber.StatusInternalServerError).JSON(panel.ErrorInternal)
	}

	return ctx.Status(fiber.StatusOK).JSON(instances)
}
