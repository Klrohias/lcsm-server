package db

import (
	"fmt"
	"os"

	"github.com/klrohias/lcsm-server/panel/models"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

type DbContext struct {
	DB *gorm.DB
}

func NewDbContext() (*DbContext, error) {
	dbPath := os.Getenv("DB_PATH")
	if dbPath == "" {
		dbPath = "./lcsm.db"
	}

	db, err := gorm.Open(sqlite.Open(dbPath), &gorm.Config{})
	if err != nil {
		return nil, fmt.Errorf("failed to connect database: %v", err)
	}

	// Migrate models
	if err := db.AutoMigrate(&models.User{}, &models.Runner{}); err != nil {
		return nil, fmt.Errorf("failed to migrate database: %v", err)
	}

	return &DbContext{
		DB: db,
	}, nil
}
