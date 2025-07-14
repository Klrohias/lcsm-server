use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PaginationOptions {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Serialize)]
pub struct PaginationResponse<T> {
    pub total: u64,
    pub page_count: u64,
    pub data: Vec<T>,
}
