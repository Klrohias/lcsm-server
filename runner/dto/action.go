package dto

import "encoding/json"

type ActionRequestPacket struct {
	Action string          `json:"action"`
	Data   json.RawMessage `json:"data"`
	Echo   string          `json:"echo"`
}

type ActionResponsePacket struct {
	Error string          `json:"error,omitempty"`
	Data  json.RawMessage `json:"data,omitempty"`
	Echo  string          `json:"echo"`
}
