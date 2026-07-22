use std::{env, fs, path::PathBuf, time::Duration};

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub fn get_data_path(service_name: &str) -> PathBuf {
    match env::var("LCSM_DATA_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => env::current_dir()
            .map(|x| x.join("data").join(service_name))
            .expect("failed to get default data path"),
    }
}

pub fn get_database_url(service_name: &str) -> String {
    if let Ok(url) = env::var("LCSM_DATABASE") {
        return url;
    }

    let data_path = get_data_path(service_name);
    fs::create_dir_all(&data_path).expect("failed to create data path");
    format!("sqlite://{}?mode=rwc", data_path.join("lcsm.db").display())
}

pub async fn build_database_connection(service_name: &str) -> DatabaseConnection {
    let database_connection_string = get_database_url(service_name);

    let mut options = ConnectOptions::new(database_connection_string);
    options
        .max_connections(100)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true);

    Database::connect(options).await.expect("failed to open db")
}
