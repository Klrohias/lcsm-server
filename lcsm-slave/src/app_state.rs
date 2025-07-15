use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::services::ProcessManagementService;

pub type AppStateRef = Arc<AppState>;

pub struct AppState {
    pub db: DatabaseConnection,
    pub pm: ProcessManagementService,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            pm: ProcessManagementService::new(),
        }
    }
}
