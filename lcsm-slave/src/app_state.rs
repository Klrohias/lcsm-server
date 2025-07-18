use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use sea_orm::DatabaseConnection;

use crate::services::{LogService, ProcessManagementService};

pub type AppStateRef = Arc<AppState>;

pub struct AppState {
    pub log_path: PathBuf,

    pub database: DatabaseConnection,
    pub process_manager: ProcessManagementService,
    pub log_manager: LogService,
}

impl AppState {
    pub fn new(database: DatabaseConnection, data_path: impl AsRef<Path>) -> Self {
        let data_path = data_path.as_ref();
        let log_path = data_path.join("logs");

        Self {
            database,
            process_manager: ProcessManagementService::new(),
            log_path: log_path.clone(),
            log_manager: LogService::new(log_path),
        }
    }

    pub fn ensure_path_created(&self) -> Result<(), io::Error> {
        if !fs::exists(&self.log_path)? {
            fs::create_dir(&self.log_path)?
        };

        Ok(())
    }
}
