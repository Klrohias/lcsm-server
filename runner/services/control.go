package services

import (
	"encoding/json"

	"github.com/gofiber/contrib/websocket"
	"github.com/klrohias/lcsm-server/common"
	"github.com/klrohias/lcsm-server/runner/dto"
	"github.com/klrohias/lcsm-server/runner/models"
	"gorm.io/gorm"
)

type ControlSocketService struct {
	authToken string
	db        *gorm.DB
	logger    common.Logger
}

func NewControlSocketService(
	db *gorm.DB,
	logger common.Logger,
) *ControlSocketService {
	return &ControlSocketService{
		db:     db,
		logger: logger,
	}
}

func (c *ControlSocketService) SetAuthToken(token string) {
	c.authToken = token
}

func (c *ControlSocketService) HandleAuthPacket(dto *dto.AuthPacket) bool {
	// Verify the auth token
	return dto.AuthToken == c.authToken
}

// HandleListInstances handles the request to list all instances with pagination
func (c *ControlSocketService) HandleListInstances(conn *websocket.Conn, action *dto.ActionRequestPacket, response *dto.ActionResponsePacket) {
	// Get page parameters from action data
	var err error
	pageDto := &dto.PageDto{}
	if err = json.Unmarshal(action.Data, pageDto); err != nil {
		c.logger.Errorf("%s", err)
		response.Error = "Invalid page parameters"
		return
	}

	// Set default values if not provided
	if pageDto.PageSize <= 0 {
		pageDto.PageSize = 10
	}
	if pageDto.Page <= 0 {
		pageDto.Page = 1
	}

	// Calculate offset
	offset := (pageDto.Page - 1) * pageDto.PageSize

	// Get total count
	var total int64
	if err := c.db.Model(&models.Instance{}).Count(&total).Error; err != nil {
		c.logger.Errorf("%s", err)
		response.Error = "Failed to count instances"
		return
	}

	// Get paginated instances
	var instances []models.Instance
	result := c.db.Offset(offset).Limit(pageDto.PageSize).Find(&instances)
	if result.Error != nil {
		c.logger.Errorf("%s", result.Error)
		response.Error = "Failed to fetch instances"
		return
	}

	// Set response data
	if response.Data, err = json.Marshal(&dto.PagedResultDto{
		Total: int(total),
		Items: instances,
	}); err != nil {
		response.Error = "Failed to generate response"
	}
}

func (c *ControlSocketService) HandleAddInstance(conn *websocket.Conn, action *dto.ActionRequestPacket, response *dto.ActionResponsePacket) {
	// Get page parameters from action data
	// var err error

	// if common.JsonUmmarshalNotNil(action.Data, , &dto.PageDto{}); err != nil {
	// 	c.logger.Errorf("%s", err)
	// 	response.Error = "Invalid page parameters"
	// 	return
	// }

}

// HandleWebSocket handles the WebSocket connection and manages the communication flow
func (c *ControlSocketService) HandleWebSocket(conn *websocket.Conn) {
	authed := false
	for {
		if authed {
			requestPacket := &dto.ActionRequestPacket{}
			if err := conn.ReadJSON(&requestPacket); err != nil {
				c.logger.Errorf("%s", err)
				break
			}

			responsePacket := &dto.ActionResponsePacket{
				Echo: requestPacket.Echo,
			}

			switch requestPacket.Action {
			case "ListInstances":
				c.HandleListInstances(conn, requestPacket, responsePacket)
			case "AddInstance":
				c.HandleAddInstance(conn, requestPacket, responsePacket)
			case "StartInstance":
				// TODO
			case "StopInstance":
				// TODO
			case "KillInstance":
				// TODO
			default:
				responsePacket.Error = "Unknown action"
			}

			conn.WriteJSON(responsePacket)
		} else {
			// Handle authentication
			authDto := &dto.AuthPacket{}
			if err := conn.ReadJSON(authDto); err != nil {
				c.logger.Errorf("%s", err)
				break
			}

			// Verify auth token
			if c.HandleAuthPacket(authDto) {
				authed = true
				conn.WriteJSON(map[string]string{"status": "Authenticated"})
			} else {
				conn.WriteJSON(map[string]string{"error": "Authentication failed"})
				break
			}
		}
	}

	// Close connection when loop ends
	conn.Close()
}
