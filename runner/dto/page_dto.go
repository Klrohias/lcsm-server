package dto

type PageDto struct {
	Page     int `json:"page"`
	PageSize int `json:"pageSize"`
}

type PagedResultDto struct {
	Total int `json:"total"`
	Items any `json:"items"`
}
