use sea_orm::DatabaseConnection;
use std::sync::Arc;
use typed_container::Container;

use crate::services::{AuthServiceRef, PermissionServiceRef, UserServiceRef};

pub type AppStateRef = Arc<AppState>;

pub struct AppState {
    pub database_connection: DatabaseConnection,
    pub auth_service: AuthServiceRef,
    pub permission_service: PermissionServiceRef,
    pub user_service: UserServiceRef,
}

impl From<Container<'_>> for AppState {
    fn from(value: Container<'_>) -> Self {
        Self {
            auth_service: value.get(),
            database_connection: value.get(),
            permission_service: value.get(),
            user_service: value.get(),
        }
    }
}
