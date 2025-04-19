package controllers

import (
	"strconv"

	"github.com/gofiber/fiber/v2"
	"github.com/klrohias/lcsm-server/common"
	"github.com/klrohias/lcsm-server/panel/db"
	"github.com/klrohias/lcsm-server/panel/models"
	"gorm.io/gorm"
)

type RunnerController struct {
	db     *gorm.DB
	logger common.Logger
}

func NewRunnerController(db *db.DbContext,
	logger common.Logger,
) *RunnerController {
	return &RunnerController{
		db:     db.DB,
		logger: logger,
	}
}

func (c *RunnerController) GetRunners(ctx *fiber.Ctx) error {
	var runners []models.Runner
	if err := c.db.Find(&runners).Error; err != nil {
		c.logger.Debugf("Error getting runners: %v", err)
		return ctx.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"error": err.Error()})
	}
	return ctx.Status(fiber.StatusOK).JSON(runners)
}

func (c *RunnerController) CreateRunner(ctx *fiber.Ctx) error {
	var runner models.Runner
	if err := ctx.BodyParser(&runner); err != nil {
		c.logger.Debugf("Error parsing runner: %v", err)
		return ctx.Status(fiber.StatusBadRequest).JSON(fiber.Map{"error": err.Error()})
	}

	if err := c.db.Create(&runner).Error; err != nil {
		c.logger.Debugf("Error creating runner: %v", err)
		return ctx.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"error": err.Error()})
	}

	return ctx.Status(fiber.StatusCreated).JSON(runner)
}

func (c *RunnerController) UpdateRunner(ctx *fiber.Ctx) error {
	id, err := strconv.ParseUint(ctx.Params("id"), 10, 32)
	if err != nil {
		c.logger.Debugf("Error parsing id: %v", err)
		return ctx.Status(fiber.StatusBadRequest).JSON(fiber.Map{"error": "invalid id"})
	}

	var runner models.Runner
	if err = c.db.First(&runner, uint(id)).Error; err != nil {
		c.logger.Debugf("Error getting runner: %v", err)
		return ctx.Status(fiber.StatusNotFound).JSON(fiber.Map{"error": "runner not found"})
	}

	// Patch
	patchData := ctx.Body()
	if err = common.ApplyJsonPatch(&runner, patchData); err != nil {
		c.logger.Debugf("Error patching runner: %v", err)
		return ctx.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"error": "jsonpatch error"})
	}

	if uint64(runner.ID) != id {
		return ctx.Status(fiber.StatusConflict).JSON(fiber.Map{"error": "id changed"})
	}

	// Save
	if err := c.db.Save(&runner).Error; err != nil {
		c.logger.Debugf("Error saving runner: %v", err)
		return ctx.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"error": "internal error"})
	}

	return ctx.Status(fiber.StatusOK).JSON(runner)
}

func (c *RunnerController) DeleteRunner(ctx *fiber.Ctx) error {
	id, err := strconv.ParseUint(ctx.Params("id"), 10, 32)
	if err != nil {
		c.logger.Debugf("Error parsing id: %v", err)
		return ctx.Status(fiber.StatusBadRequest).JSON(fiber.Map{"error": "invalid id"})
	}

	var runner models.Runner
	if err := c.db.First(&runner, uint(id)).Error; err != nil {
		c.logger.Debugf("Error getting runner: %v", err)
		return ctx.Status(fiber.StatusNotFound).JSON(fiber.Map{"error": "runner not found"})
	}

	if err := c.db.Delete(&runner).Error; err != nil {
		c.logger.Debugf("Error deleting runner: %v", err)
		return ctx.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"error": "internal error"})
	}

	return ctx.Status(fiber.StatusOK).JSON(fiber.Map{"message": "Runner deleted successfully"})
}
