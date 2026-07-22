use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct PaginationOptions {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

impl PaginationOptions {
    pub fn normalized(&self) -> (u64, u64) {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(10).max(1);

        (page, page_size)
    }
}

#[derive(Serialize)]
pub struct PaginationResponse<T> {
    pub total: u64,
    pub page_count: u64,
    pub data: Vec<T>,
}
