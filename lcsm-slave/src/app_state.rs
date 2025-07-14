use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::ProcessManager;

pub type AppStateRef = Arc<AppState>;

pub struct AppState {
    pub db: DatabaseConnection,
    pub pm: ProcessManager,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            pm: ProcessManager::new(),
        }
    }
}
