package client

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"math/rand"
	"time"

	"github.com/gorilla/websocket"
	"github.com/klrohias/lcsm-server/common"
	"github.com/klrohias/lcsm-server/runner/dto"
)

type pendingRequest struct {
	*dto.ActionResponsePacket
	error
}

type workerCtx struct {
	context.Context
	context.CancelFunc
}

type Client struct {
	conn      *websocket.Conn
	workerCtx *workerCtx

	endPoint  string
	authToken string
	timeout   time.Duration
	logger    common.Logger

	pendingRequests map[string]chan pendingRequest
}

// NewClient creates a new instance of the client
func NewClient(endPoint string, authToken string, logger common.Logger) *Client {
	return &Client{
		endPoint:        endPoint,
		authToken:       authToken,
		timeout:         time.Duration(30) * time.Second,
		logger:          logger,
		pendingRequests: make(map[string]chan pendingRequest),
	}
}

func (c *Client) getActiveConn() (*websocket.Conn, error) {
	if c.conn != nil {
		return c.conn, nil
	}

	conn, _, err := websocket.DefaultDialer.Dial(c.endPoint, nil)
	if err != nil {
		return nil, err
	}

	conn.SetCloseHandler(func(code int, text string) error {
		c.conn = nil
		if c.workerCtx != nil {
			c.workerCtx.CancelFunc()
			c.workerCtx = nil
		}

		return nil
	})

	// Perform authentication
	if err := c.authenticate(conn); err != nil {
		conn.Close()
		return nil, err
	}

	// Start worker
	ctx, cancel := context.WithCancel(context.Background())
	c.workerCtx = &workerCtx{ctx, cancel}
	go c.worker(ctx)

	c.conn = conn

	return conn, nil
}

// authenticate performs the initial authentication with the server
func (c *Client) authenticate(conn *websocket.Conn) error {
	authDto := &dto.AuthPacket{
		AuthToken: c.authToken,
	}

	if err := conn.WriteJSON(authDto); err != nil {
		return err
	}

	response := make(map[string]string)
	if err := conn.ReadJSON(&response); err != nil {
		return err
	}

	if response["error"] != "" {
		return errors.New(response["error"])
	}

	return nil
}

// generateEcho generates a random echo string for action requests
func (c *Client) generateEcho() string {
	return fmt.Sprintf("%d", rand.Int63())
}

func (c *Client) worker(ctx context.Context) {
	for {
		select {
		case <-ctx.Done():
			{
				break
			}
		default:
			{
				// Receive packet and process
				conn := c.conn
				if conn == nil {
					break
				}

				response := &dto.ActionResponsePacket{}
				if err := conn.ReadJSON(response); err != nil {
					c.logger.Errorf("%s", err)
					continue
				}

				// Check echo
				if receiveChan, ok := c.pendingRequests[response.Echo]; ok {
					receiveChan <- pendingRequest{response, nil}
				} else {
					c.logger.Warnf("Packet with echo %s not found, ignored", response.Echo)
				}
			}
		}
	}
}

func (c *Client) SetTimeout(duration time.Duration) {
	c.timeout = duration
}

func (c *Client) Timeout() time.Duration {
	return c.timeout
}

func (c *Client) InvokeAction(action string, data any) (*dto.ActionResponsePacket, error) {
	var activeConn *websocket.Conn
	var err error

	if activeConn, err = c.getActiveConn(); err != nil {
		return nil, err
	}

	// prepare packet
	var marshaledData json.RawMessage
	marshaledData, err = json.Marshal(data)
	if err != nil {
		return nil, err
	}

	echo := c.generateEcho()
	request := &dto.ActionRequestPacket{
		Action: action,
		Data:   marshaledData,
		Echo:   echo,
	}

	// prepare receive
	receiveChan := make(chan pendingRequest)
	time.AfterFunc(c.timeout, func() { receiveChan <- pendingRequest{nil, errors.New("API client timeout")} })
	c.pendingRequests[echo] = receiveChan
	defer delete(c.pendingRequests, echo)

	// send
	if err = activeConn.WriteJSON(request); err != nil {
		return nil, err
	}

	// receive
	response := <-receiveChan
	if response.error != nil {
		return nil, response.error
	}

	if response.ActionResponsePacket.Error != "" {
		return nil, errors.New(response.ActionResponsePacket.Error)
	}

	return response.ActionResponsePacket, nil
}

// GetInstances retrieves a paginated list of instances from the server
func (c *Client) GetInstances(page, pageSize int) (*dto.PagedResultDto, error) {
	response, err := c.InvokeAction("ListInstances", &dto.PageDto{
		Page:     page,
		PageSize: pageSize,
	})

	if err != nil {
		return nil, err
	}

	result := &dto.PagedResultDto{}
	if err = json.Unmarshal(response.Data, result); err != nil {
		return nil, err
	}

	return result, nil
}

// StartInstance sends a request to start a specific instance
func (c *Client) StartInstance(instanceID uint) error {
	_, err := c.InvokeAction("StartInstance", instanceID)
	return err
}

// StopInstance sends a request to stop a specific instance
func (c *Client) StopInstance(instanceID uint) error {
	_, err := c.InvokeAction("StopInstance", instanceID)
	return err
}

// KillInstance sends a request to forcefully terminate a specific instance
func (c *Client) KillInstance(instanceID uint) error {
	_, err := c.InvokeAction("KillInstance", instanceID)
	return err
}

// Close closes the WebSocket connection
func (c *Client) Close() error {
	return c.conn.Close()
}
