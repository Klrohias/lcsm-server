use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use sea_orm::DatabaseConnection;

use crate::services::{LogService, ProcessManagementService};

pub type AppStateRef = Arc<AppState>;

pub struct AppState {
    pub data_path: PathBuf,
    pub instances_path: PathBuf,
    pub log_path: PathBuf,

    pub database: DatabaseConnection,
    pub process_manager: ProcessManagementService,
    pub log_manager: LogService,
}

impl AppState {
    pub fn new(database: DatabaseConnection, data_path: impl AsRef<Path>) -> Self {
        let data_path = data_path.as_ref().to_path_buf();
        let instances_path = data_path.join("instances");
        let log_path = data_path.join("logs");

        Self {
            data_path,
            instances_path,
            database,
            process_manager: ProcessManagementService::new(),
            log_path: log_path.clone(),
            log_manager: LogService::new(log_path),
        }
    }

    pub fn ensure_path_created(&self) -> Result<(), io::Error> {
        for path in [&self.data_path, &self.instances_path, &self.log_path] {
            if !fs::exists(path)? {
                fs::create_dir_all(path)?
            }
        }

        Ok(())
    }
}
