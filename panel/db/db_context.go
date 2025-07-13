package db

import (
	"fmt"
	"os"

	"github.com/klrohias/lcsm-server/panel/models"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

func NewDbContext() (*gorm.DB, error) {
	dbPath := os.Getenv("LCSM_DB_PATH")
	if dbPath == "" {
		dbPath = "./lcsm.db"
	}

	db, err := gorm.Open(sqlite.Open(dbPath), &gorm.Config{})
	if err != nil {
		return nil, fmt.Errorf("failed to connect database: %v", err)
	}

	// Migrate models
	if err := db.AutoMigrate(&models.User{}, &models.Runner{}, &models.Permission{}); err != nil {
		return nil, fmt.Errorf("failed to migrate database: %v", err)
	}

	return db, nil
}
