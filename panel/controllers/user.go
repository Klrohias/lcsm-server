package controllers

import (
	"github.com/gofiber/fiber/v2"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"

	"github.com/klrohias/lcsm-server/common"
	"github.com/klrohias/lcsm-server/panel/auth"
	"github.com/klrohias/lcsm-server/panel/db"
	"github.com/klrohias/lcsm-server/panel/models"
)

type UserController struct {
	db         *gorm.DB
	jwtContext *auth.JwtContext
	logger     common.Logger
}

func NewUserController(db *db.DbContext,
	jwtContext *auth.JwtContext,
	logger common.Logger,
) *UserController {
	return &UserController{
		db:         db.DB,
		jwtContext: jwtContext,
		logger:     logger,
	}
}

type AuthenticateRequest struct {
	Username string `json:"username" binding:"required"`
	Password string `json:"password" binding:"required"`
}

type UserResponse struct {
	Username string `json:"username"`
	Nickname string `json:"nickname"`
	Role     string `json:"role"`
	ID       uint   `json:"id"`
}

type UserUpdateRequest struct {
	Username string `json:"username"`
	Password string `json:"password"`
	Role     string `json:"role"`
}

func (uc *UserController) CurrentUser(c *fiber.Ctx) error {
	// Get claims from context
	claims := auth.GetClaims(c)
	if claims == nil {
		return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{"error": "Unauthorized"})
	}

	// Find user
	var user models.User
	if result := uc.db.Where("username = ?", claims.Username).First(&user); result.Error != nil {
		uc.logger.Debugf("User not found: %v", result.Error)
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"error": "User not found"})
	}

	// Return user info without sensitive fields
	return c.Status(fiber.StatusOK).JSON(UserResponse{
		Username: user.Username,
		Nickname: user.Nickname,
		Role:     string(user.Role),
		ID:       user.ID,
	})
}

func (uc *UserController) Authenticate(c *fiber.Ctx) error {
	var req AuthenticateRequest
	if err := c.BodyParser(&req); err != nil {
		uc.logger.Debugf("Invalid request: %v", err)
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{"error": "Invalid request"})
	}

	// Find user
	var user models.User
	if result := uc.db.Where("username = ?", req.Username).First(&user); result.Error != nil {
		uc.logger.Debugf("User not found: %v", result.Error)
		return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{"error": "Invalid credentials"})
	}

	// Check password
	if err := bcrypt.CompareHashAndPassword([]byte(user.Password), []byte(req.Password)); err != nil {
		uc.logger.Debugf("Invalid password: %v", err)
		return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{"error": "Invalid credentials"})
	}

	// Generate JWT token
	token, err := uc.jwtContext.GenerateToken(user.Username, string(user.Role))
	if err != nil {
		uc.logger.Debugf("Failed to generate token: %v", err)
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"error": "Failed to generate token"})
	}

	return c.Status(fiber.StatusOK).JSON(fiber.Map{
		"accessToken": token,
	})
}

func (uc *UserController) CreateUser(c *fiber.Ctx) error {
	var req UserUpdateRequest
	if err := c.BodyParser(&req); err != nil {
		uc.logger.Debugf("Invalid request: %v", err)
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{"error": "Invalid request"})
	}

	// Check if user exists
	var userCount int64
	if result := uc.db.Model(&models.User{}).Where("username = ?", req.Username).Count(&userCount); result.Error != nil {
		uc.logger.Debugf("Failed to count users: %v", result.Error)
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"error": "Internal error"})
	}

	if userCount != 0 {
		return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{"error": "User existed"})
	}

	// Check if it should be a admin
	if result := uc.db.Model(&models.User{}).Count(&userCount); result.Error != nil {
		uc.logger.Debugf("Failed to count users: %v", result.Error)
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"error": "Internal error"})
	}

	shouldBeAdmin := req.Role == string(models.RoleAdmin)

	if userCount == 0 {
		// It is the first user
		shouldBeAdmin = true
	}

	// Hash password
	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(req.Password), bcrypt.DefaultCost)
	if err != nil {
		uc.logger.Debugf("Failed to hash password: %v", err)
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"error": "Failed to hash password"})
	}

	// Create user
	user := models.User{
		Username: req.Username,
		Password: string(hashedPassword),
		Nickname: req.Username, // Use username as default nickname
		Role:     models.UserRole(models.RoleAdmin),
	}

	if !shouldBeAdmin {
		user.Role = models.UserRole(models.RoleUser)
	}

	if result := uc.db.Create(&user); result.Error != nil {
		uc.logger.Debugf("Failed to create user: %v", result.Error)
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{"error": "Failed to create user"})
	}

	return c.Status(fiber.StatusOK).JSON(UserResponse{
		Username: user.Username,
		Nickname: user.Nickname,
		Role:     string(user.Role),
		ID:       user.ID,
	})
}
