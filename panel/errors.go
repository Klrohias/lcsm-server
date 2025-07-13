package panel

import "github.com/gofiber/fiber/v2"

var (
	ErrorInvalidRunnerID = errorMessage("Invalid runner ID")
	ErrorInvalidBody     = errorMessage("Invalid request body")
	ErrorInternal        = errorMessage("Internal server error")
	ErrorUnauthorized    = errorMessage("Unauthorized")
	ErrorAlreadyExisted  = errorMessage("Exists")
	ErrorForbidden       = errorMessage("Forbidden")
)

func errorMessage(msg string) fiber.Map {
	return fiber.Map{"message": msg}
}
