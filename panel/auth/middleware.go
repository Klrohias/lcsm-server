package auth

import (
	"strings"

	"github.com/gofiber/fiber/v2"
)

const claimsKey string = "claims"

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

func JwtAuthMiddleware(jwt *JwtContext) fiber.Handler {
	return func(c *fiber.Ctx) error {
		authorization := c.Get("Authorization")
		if authorization == "" {
			return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{"error": "Authorization header is required"})
		}

		parts := strings.Split(authorization, " ")
		if len(parts) != 2 || parts[0] != "Bearer" {
			return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{"error": "Invalid authorization format"})
		}

		tokenString := parts[1]
		claims, err := jwt.ParseToken(tokenString)
		if err != nil {
			return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{"error": "Invalid or expired token"})
		}

		// Save claims in the request context
		PutClaims(c, claims)

		return c.Next()
	}
}

func AdminRoleMiddleware() fiber.Handler {
	return func(c *fiber.Ctx) error {
		claims := GetClaims(c)
		if claims == nil || claims.Role != "admin" {
			return c.Status(fiber.StatusForbidden).JSON(fiber.Map{"error": "Admin privileges required"})
		}
		return c.Next()
	}
}
