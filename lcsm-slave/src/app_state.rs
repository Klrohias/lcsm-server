use std::sync::Arc;

use sea_orm::DatabaseConnection;

pub type AppStateRef = Arc<AppState>;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
