package db

import (
	"fmt"
	"os"

	"github.com/klrohias/lcsm-server/runner/models"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

func NewDbContext() (*gorm.DB, error) {
	dbPath := os.Getenv("LCSM_DB_PATH")
	if dbPath == "" {
		dbPath = "./lcsm-node.db"
	}

	db, err := gorm.Open(sqlite.Open(dbPath), &gorm.Config{})
	if err != nil {
		return nil, fmt.Errorf("failed to connect database: %v", err)
	}

	// Migrate models
	if err := db.AutoMigrate(&models.Instance{}); err != nil {
		return nil, fmt.Errorf("failed to migrate database: %v", err)
	}

	return db, nil
}
