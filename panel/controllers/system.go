package controllers

import (
	"github.com/gin-gonic/gin"
	"github.com/klrohias/lcsm-server/panel/db"
	"github.com/klrohias/lcsm-server/panel/models"
	"gorm.io/gorm"
)

type SystemController struct {
	db *gorm.DB
}

type SystemHealthResponse struct {
	TotalUsers int `json:"totalUser"`
}

func NewSystemController(db *db.DbContext) *SystemController {
	return &SystemController{db: db.DB}
}

func (sc *SystemController) SystemHealth(c *gin.Context) {
	// Report total user count
	var totalUsers int64
	if result := sc.db.Model(&models.User{}).Count(&totalUsers); result.Error != nil {
		c.JSON(500, gin.H{"error": "Failed to count users"})
		return
	}

	c.JSON(200, SystemHealthResponse{
		TotalUsers: int(totalUsers),
	})
}
