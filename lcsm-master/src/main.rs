use std::{env, sync::Arc, time::Duration};

use axum::Router;
use lcsm_master::{
    AppState, AppStateRef, routes,
    services::{AuthService, AuthServiceRef},
};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

async fn build_database_connection() -> DatabaseConnection {
    let database_connection_string = env::var("LCSM_DATABASE").expect("LCSM_DATABASE is missing");

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

    Database::connect(options).await.expect("failed to open db")
}

fn build_auth_service() -> AuthServiceRef {
    AuthService::new(env::var("LCSM_JWT_SECRET").expect("LCSM_JWT_SECRET is missing"))
}

fn build_service(app: Router) -> Router {
    app.layer(ServiceBuilder::new().layer(CorsLayer::new()))
}

fn build_routes(app: Router, state: &AppStateRef) -> Router {
    app.merge(routes::get_routes(state))
}

async fn build_app() -> Router {
    // build state
    let app_state = Arc::new(AppState::new(
        build_database_connection().await,
        build_auth_service(),
    ));

    // build app
    let app = Router::new();
    let app = build_routes(app, &app_state);
    let app = build_service(app);

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
        .expect("failed to bind address");
    let app = build_app().await;
    axum::serve(listener, app)
        .await
        .expect("failed to serve app");
}
