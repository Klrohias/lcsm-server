package services

import (
	"fmt"
	"strings"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/golang-jwt/jwt/v5"
	"github.com/klrohias/lcsm-server/panel"
)

type AuthService struct {
	secret []byte
}

func NewAuthService() *AuthService {
	return &AuthService{}
}

const claimsKey string = "claims"

type Claims struct {
	Username string `json:"username"`
	Role     string `json:"role"`
	jwt.RegisteredClaims
}

func PutClaims(c *fiber.Ctx, claims *Claims) {
	c.Context().SetUserValue(claimsKey, claims)
}

func GetClaims(c *fiber.Ctx) *Claims {
	obj, ok := c.Context().UserValue(claimsKey).(*Claims)
	if !ok {
		return nil
	}

	return obj
}

func (a *AuthService) SetJwtSecret(secret string) {
	a.secret = []byte(secret)
}

func (a *AuthService) GenerateToken(username, role string) (string, error) {
	now := time.Now()
	claims := Claims{
		Username: username,
		Role:     role,
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(now.Add(24 * time.Hour)),
			IssuedAt:  jwt.NewNumericDate(now),
			NotBefore: jwt.NewNumericDate(now),
		},
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return token.SignedString(a.secret)
}

func (a *AuthService) ParseToken(tokenString string) (*Claims, error) {
	token, err := jwt.ParseWithClaims(tokenString, &Claims{}, func(token *jwt.Token) (interface{}, error) {
		if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
		return a.secret, nil
	})

	if err != nil {
		return nil, err
	}

	if claims, ok := token.Claims.(*Claims); ok && token.Valid {
		return claims, nil
	}

	return nil, fmt.Errorf("invalid token")
}

func (a *AuthService) JwtAuthMiddleware() fiber.Handler {
	return func(c *fiber.Ctx) error {
		authorization := c.Get("Authorization")
		if authorization == "" {
			return c.Status(fiber.StatusUnauthorized).JSON(panel.ErrorUnauthorized)
		}

		parts := strings.Split(authorization, " ")
		if len(parts) != 2 || parts[0] != "Bearer" {
			return c.Status(fiber.StatusUnauthorized).JSON(panel.ErrorUnauthorized)
		}

		tokenString := parts[1]
		claims, err := a.ParseToken(tokenString)
		if err != nil {
			return c.Status(fiber.StatusUnauthorized).JSON(panel.ErrorUnauthorized)
		}

		// Save claims in the request context
		PutClaims(c, claims)

		return c.Next()
	}
}

func (a *AuthService) AdminRoleMiddleware() fiber.Handler {
	return func(c *fiber.Ctx) error {
		claims := GetClaims(c)
		if claims == nil || claims.Role != "admin" {
			return c.Status(fiber.StatusForbidden).JSON(panel.ErrorForbidden)
		}
		return c.Next()
	}
}
