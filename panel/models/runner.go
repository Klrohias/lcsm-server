package models

import (
	"time"

	"gorm.io/gorm"
)

type RunnerType string

const (
	RunnerTypeBuiltin   RunnerType = "builtin"
	RunnerTypeWebsocket RunnerType = "websocket"
)

type Runner struct {
	ID        uint           `gorm:"primarykey" json:"id"`
	CreatedAt time.Time      `json:"created_at"`
	UpdatedAt time.Time      `json:"updated_at"`
	DeletedAt gorm.DeletedAt `gorm:"index" json:"deleted_at"`

	Type        RunnerType `gorm:"type:string;not null" json:"type"`
	Endpoint    string     `gorm:"not null" json:"endpoint"`
	Name        string     `gorm:"not null" json:"name"`
	Description string     `json:"description"`
}
