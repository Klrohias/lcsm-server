package models

import (
	"time"

	"gorm.io/gorm"
)

type Instance struct {
	ID        uint           `gorm:"primarykey" json:"id"`
	CreatedAt time.Time      `json:"created_at"`
	UpdatedAt time.Time      `json:"updated_at"`
	DeletedAt gorm.DeletedAt `gorm:"index" json:"deleted_at"`

	Name          string `gorm:"not null" json:"name"`
	Description   string `json:"description"`
	LaunchCommand string `gorm:"not null" json:"launchCommand"`
}
