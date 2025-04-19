package models

import (
	"time"

	"gorm.io/gorm"
)

type UserRole string

const (
	RoleUser  UserRole = "user"
	RoleAdmin UserRole = "admin"
)

type User struct {
	ID        uint `gorm:"primarykey"`
	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt gorm.DeletedAt `gorm:"index"`

	Username string   `gorm:"uniqueIndex;not null"`
	Password string   `gorm:"not null"`
	Nickname string   `gorm:"not null"`
	Role     UserRole `gorm:"type:string;default:'user'"`
}
