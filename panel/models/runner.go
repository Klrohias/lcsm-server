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
	CreatedAt time.Time      `json:"createdAt"`
	UpdatedAt time.Time      `json:"updatedAt"`
	DeletedAt gorm.DeletedAt `gorm:"index" json:"deletedAt"`

	Type        RunnerType `gorm:"type:string;not null" json:"type"`
	EndPoint    string     `json:"endPoint"`
	Name        string     `gorm:"not null" json:"name"`
	AuthToken   string     `json:"authToken"`
	Description string     `json:"description"`
}
