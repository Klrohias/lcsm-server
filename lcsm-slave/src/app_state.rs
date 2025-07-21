use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use sea_orm::DatabaseConnection;

#[cfg(feature = "docker")]
use crate::services::DockerService;
use crate::services::{ProcessLogService, ProcessManagementService};

pub type AppStateRef = Arc<AppState>;

pub struct AppState {
    pub log_path: PathBuf,

    pub database: DatabaseConnection,
    pub process_manager: ProcessManagementService,
    pub log_manager: ProcessLogService,

    #[cfg(feature = "docker")]
    pub docker_service: DockerService,
}

impl AppState {
    pub fn new(database: DatabaseConnection, data_path: impl AsRef<Path>) -> Self {
        let data_path = data_path.as_ref();
        let log_path = data_path.join("logs");

        let log_manager = ProcessLogService::new(log_path.clone());
        let process_manager = ProcessManagementService::new();

        #[cfg(feature = "docker")]
        let docker_service = DockerService::new();

        Self {
            database,
            process_manager,
            log_path,
            log_manager,

            #[cfg(feature = "docker")]
            docker_service,
        }
    }

    pub fn ensure_path_created(&self) -> Result<(), io::Error> {
        if !fs::exists(&self.log_path)? {
            fs::create_dir(&self.log_path)?
        };

        Ok(())
    }
}
