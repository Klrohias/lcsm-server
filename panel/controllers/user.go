package controllers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"

	"github.com/klrohias/lcsm-server/panel/auth"
	"github.com/klrohias/lcsm-server/panel/db"
	"github.com/klrohias/lcsm-server/panel/models"
)

type UserController struct {
	db         *gorm.DB
	jwtContext *auth.JwtContext
}

func NewUserController(db *db.DbContext, jwtContext *auth.JwtContext) *UserController {
	return &UserController{db: db.DB, jwtContext: jwtContext}
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

func (uc *UserController) CurrentUser(c *gin.Context) {
	// Get claims from context
	claims := auth.GetClaimsFromContext(c.Request.Context())
	if claims == nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Unauthorized"})
		return
	}

	// Find user
	var user models.User
	if result := uc.db.Where("username = ?", claims.Username).First(&user); result.Error != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "User not found"})
		return
	}

	// Return user info without sensitive fields
	c.JSON(http.StatusOK, UserResponse{
		Username: user.Username,
		Nickname: user.Nickname,
		Role:     string(user.Role),
		ID:       user.ID,
	})
}

func (uc *UserController) Authenticate(c *gin.Context) {
	var req AuthenticateRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	// Find user
	var user models.User
	if result := uc.db.Where("username = ?", req.Username).First(&user); result.Error != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid credentials"})
		return
	}

	// Check password
	if err := bcrypt.CompareHashAndPassword([]byte(user.Password), []byte(req.Password)); err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid credentials"})
		return
	}

	// Generate JWT token
	token, err := uc.jwtContext.GenerateToken(user.Username, string(user.Role))
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to generate token"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"accessToken": token,
	})
}

func (uc *UserController) CreateUser(c *gin.Context) {
	var req UserUpdateRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	// Check if user exists
	var userCount int64
	if result := uc.db.Model(&models.User{}).Where("username = ?", req.Username).Count(&userCount); result.Error != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Internal error"})
		return
	}

	if userCount != 0 {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User existed"})
		return
	}

	// Check if it should be a admin
	if result := uc.db.Model(&models.User{}).Count(&userCount); result.Error != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Internal error"})
		return
	}

	shouldBeAdmin := req.Role == string(models.RoleAdmin)

	if userCount == 0 {
		// It is the first user
		shouldBeAdmin = true
	}

	// Hash password
	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(req.Password), bcrypt.DefaultCost)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to hash password"})
		return
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
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create user"})
		return
	}

	c.JSON(http.StatusOK, UserResponse{
		Username: user.Username,
		Nickname: user.Nickname,
		Role:     string(user.Role),
		ID:       user.ID,
	})
}
