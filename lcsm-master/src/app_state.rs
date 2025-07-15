use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::services::AuthServiceRef;

pub type AppStateRef = Arc<AppState>;

pub struct AppState {
    pub database_connection: DatabaseConnection,
    pub auth_service: AuthServiceRef,
}

impl AppState {
    pub fn new(database_connection: DatabaseConnection, auth_service: AuthServiceRef) -> Self {
        Self {
            database_connection,
            auth_service,
        }
    }
}
