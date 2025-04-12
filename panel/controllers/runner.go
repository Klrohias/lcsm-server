package controllers

import (
	"net/http"
	"strconv"

	"github.com/klrohias/lcsm-server/panel/db"
	"github.com/klrohias/lcsm-server/panel/models"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

type RunnerController struct {
	db *gorm.DB
}

func NewRunnerController(db *db.DbContext) *RunnerController {
	return &RunnerController{db: db.DB}
}

func (c *RunnerController) GetRunners(ctx *gin.Context) {
	var runners []models.Runner
	if err := c.db.Find(&runners).Error; err != nil {
		ctx.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	ctx.JSON(http.StatusOK, runners)
}

func (c *RunnerController) CreateRunner(ctx *gin.Context) {
	var runner models.Runner
	if err := ctx.ShouldBindJSON(&runner); err != nil {
		ctx.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := c.db.Create(&runner).Error; err != nil {
		ctx.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	ctx.JSON(http.StatusCreated, runner)
}

func (c *RunnerController) UpdateRunner(ctx *gin.Context) {
	id, err := strconv.ParseUint(ctx.Param("id"), 10, 32)
	if err != nil {
		ctx.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"})
		return
	}

	var runner models.Runner
	if err := c.db.First(&runner, uint(id)).Error; err != nil {
		ctx.JSON(http.StatusNotFound, gin.H{"error": "runner not found"})
		return
	}

	var patches []map[string]interface{}
	if err := ctx.ShouldBindJSON(&patches); err != nil {
		ctx.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	for _, patch := range patches {
		op := patch["op"].(string)
		path := patch["path"].(string)
		value := patch["value"]

		switch op {
		case "replace":
			switch path {
			case "/type":
				runner.Type = models.RunnerType(value.(string))
			case "/endpoint":
				runner.Endpoint = value.(string)
			case "/name":
				runner.Name = value.(string)
			case "/description":
				runner.Description = value.(string)
			default:
				ctx.JSON(http.StatusBadRequest, gin.H{"error": "invalid path"})
				return
			}
		default:
			ctx.JSON(http.StatusBadRequest, gin.H{"error": "unsupported operation"})
			return
		}
	}

	if err := c.db.Save(&runner).Error; err != nil {
		ctx.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	ctx.JSON(http.StatusOK, runner)
}

func (c *RunnerController) DeleteRunner(ctx *gin.Context) {
	id, err := strconv.ParseUint(ctx.Param("id"), 10, 32)
	if err != nil {
		ctx.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"})
		return
	}

	var runner models.Runner
	if err := c.db.First(&runner, uint(id)).Error; err != nil {
		ctx.JSON(http.StatusNotFound, gin.H{"error": "runner not found"})
		return
	}

	if err := c.db.Delete(&runner).Error; err != nil {
		ctx.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	ctx.JSON(http.StatusOK, gin.H{"message": "runner deleted"})
}
