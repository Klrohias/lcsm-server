use std::{env, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Error, Result};
use axum::Router;
use lcsm_slave::{AppState, AppStateRef, routes};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, validate_request::ValidateRequestHeaderLayer};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

async fn build_database_connection() -> Result<DatabaseConnection> {
    let database_connection_string = env::var("LCSM_DATABASE")?;
    let options = {
        let mut options = ConnectOptions::new(database_connection_string);
        options
            .max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .sqlx_logging(true);
        options
    };

    Ok(Database::connect(options).await?)
}

fn build_services(app: Router) -> Router {
    let token = env::var("LCSM_SLAVE_TOKEN").expect("LCSM_SLAVE_TOKEN is missing");

    app.layer(
        ServiceBuilder::new()
            .layer(CorsLayer::new())
            .layer(ValidateRequestHeaderLayer::bearer(&token)),
    )
}

fn build_routes(app: Router, state: &AppStateRef) -> Router {
    app.merge(routes::get_routes(state))
}

fn get_data_path() -> Result<PathBuf, Error> {
    Ok(match env::var("LCSM_DATA_PATH") {
        Ok(path) => PathBuf::from(&path),
        Err(_) => env::current_dir()?,
    })
}

async fn build_app() -> Router {
    // build state
    let app_state = Arc::new(AppState::new(
        build_database_connection()
            .await
            .expect("build db connection"),
        get_data_path().expect("get data path"),
    ));

    app_state
        .ensure_path_created()
        .expect("ensure path created");

    // build app
    let app = Router::new();
    let app = build_routes(app, &app_state);
    let app = build_services(app);

    app
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();
}

#[tokio::main]
async fn main() {
    init_tracing();
    let listen_addr = env::var("LCSM_LISTEN_ADDR").expect("LCSM_LISTEN_ADDR is missing");
    let listener = TcpListener::bind(listen_addr)
        .await
        .expect("tcp server bind");
    let app = build_app().await;
    axum::serve(listener, app).await.expect("serve app");
}
