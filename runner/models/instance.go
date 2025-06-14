package models

import (
	"time"

	"gorm.io/gorm"
)

type Instance struct {
	ID        uint           `gorm:"primarykey" json:"id"`
	CreatedAt time.Time      `json:"createdAt"`
	UpdatedAt time.Time      `json:"updatedAt"`
	DeletedAt gorm.DeletedAt `gorm:"index" json:"deletedAt"`

	Name          string `gorm:"not null" json:"name"`
	Description   string `json:"description"`
	LaunchCommand string `gorm:"not null" json:"launchCommand"`
}
