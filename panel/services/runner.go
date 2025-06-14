package services

import (
	"github.com/klrohias/lcsm-server/common"
	"github.com/klrohias/lcsm-server/panel/db"
	"github.com/klrohias/lcsm-server/panel/models"
	"gorm.io/gorm"

	runnerClient "github.com/klrohias/lcsm-server/runner/client"
)

type ClientMap map[uint]*runnerClient.Client

type RunnerService struct {
	db      *gorm.DB
	clients ClientMap
	logger  common.Logger
}

func NewRunnerService(db *db.DbContext, logger common.Logger) *RunnerService {
	return &RunnerService{
		db:      db.DB,
		clients: make(ClientMap),
		logger:  logger,
	}
}

func (r *RunnerService) NewClient(id uint) (*runnerClient.Client, error) {
	runner := &models.Runner{}
	if result := r.db.Where("ID = ?", id).First(runner); result.Error != nil {
		return nil, result.Error
	}

	client := runnerClient.NewClient(runner.EndPoint, runner.AuthToken, r.logger)
	return client, nil
}

func (r *RunnerService) GetClient(id uint) (*runnerClient.Client, error) {
	if client, ok := r.clients[id]; ok {
		return client, nil
	}

	// New client
	client, err := r.NewClient(id)
	if err != nil {
		return nil, err
	}
	r.clients[id] = client

	return client, nil
}

func (r *RunnerService) CloseClient(id uint) {
	if client, ok := r.clients[id]; ok {
		client.Close()
		delete(r.clients, id)
	}
}
