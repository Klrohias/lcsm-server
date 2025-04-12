package client

import "github.com/klrohias/lcsm-server/runner/models"

type Client struct {
}

func NewClient() *Client {
	return &Client{}
}

func (c *Client) GetInstances() ([]models.Instance, error) {
	return nil, nil // TODO
}
