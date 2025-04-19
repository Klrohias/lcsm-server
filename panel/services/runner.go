package services

import (
	"github.com/klrohias/lcsm-server/panel/db"
	"gorm.io/gorm"

	runnerClient "github.com/klrohias/lcsm-server/runner/client"
)

type ClientMap map[uint]*runnerClient.Client

type RunnerService struct {
	db      *gorm.DB
	clients ClientMap
}

func NewRunnerService(db *db.DbContext) *RunnerService {
	return &RunnerService{
		db:      db.DB,
		clients: make(ClientMap),
	}
}

func (r *RunnerService) GetClient(id uint) (*runnerClient.Client, error) {
	if client, ok := r.clients[id]; ok {
		return client, nil
	}

	// New client
	client := runnerClient.NewClient()
	r.clients[id] = client

	return client, nil
}

func (r *RunnerService) CloseClient(id uint) {
	if _, ok := r.clients[id]; ok {
		delete(r.clients, id)
	}
}
