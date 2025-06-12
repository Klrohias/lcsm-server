package models

type Permission struct {
	ID         uint `gorm:"primarykey"`
	UserID     *uint
	GroupID    *uint
	Permission string `gorm:"not null"`
}
