package controllers

import (
	"github.com/gofiber/fiber/v2"
	"github.com/klrohias/lcsm-server/common"
	"github.com/klrohias/lcsm-server/panel"
	"github.com/klrohias/lcsm-server/panel/models"
	"gorm.io/gorm"
)

type SystemController struct {
	db     *gorm.DB
	logger common.Logger
}

type SystemHealthResponse struct {
	TotalUsers int `json:"totalUser"`
}

func NewSystemController(
	db *gorm.DB,
	logger common.Logger,
) *SystemController {
	return &SystemController{
		db,
		logger,
	}
}

func (sc *SystemController) SystemHealth(c *fiber.Ctx) error {
	// Report total user count
	var totalUsers int64
	if result := sc.db.Model(&models.User{}).Count(&totalUsers); result.Error != nil {
		sc.logger.Debugf("Failed to count users: %v", result.Error)
		return c.Status(fiber.StatusInternalServerError).JSON(panel.ErrorInternal)
	}

	return c.Status(fiber.StatusOK).JSON(SystemHealthResponse{
		TotalUsers: int(totalUsers),
	})
}
