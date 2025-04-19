package controllers

import (
	"github.com/gofiber/fiber/v2"
	"github.com/klrohias/lcsm-server/common"
	"github.com/klrohias/lcsm-server/panel/db"
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

func NewSystemController(db *db.DbContext,
	logger common.Logger,
) *SystemController {
	return &SystemController{
		db:     db.DB,
		logger: logger,
	}
}

func (sc *SystemController) SystemHealth(c *fiber.Ctx) error {
	// Report total user count
	var totalUsers int64
	if result := sc.db.Model(&models.User{}).Count(&totalUsers); result.Error != nil {
		sc.logger.Debugf("Failed to count users: %v", result.Error)
		return c.Status(500).JSON(fiber.Map{"error": "Failed to count users"})
	}

	return c.Status(200).JSON(SystemHealthResponse{
		TotalUsers: int(totalUsers),
	})
}
