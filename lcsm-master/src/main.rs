use std::{env, sync::Arc, time::Duration};

use anyhow::Result;
use axum::{Router, middleware};
use lcsm_master::{
    AppState, AppStateRef, routes,
    services::{self, AuthService, AuthServiceRef},
};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

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

fn build_auth_service() -> Result<AuthServiceRef> {
    Ok(AuthService::new(env::var("LCSM_JWT_SECRET")?))
}

fn build_service(app: Router, state: &AppStateRef) -> Result<Router> {
    Ok(
        app.layer(ServiceBuilder::new().layer(CorsLayer::new()).layer(
            middleware::from_fn_with_state(state.auth_service.clone(), services::jwt_middleware),
        )),
    )
}

fn build_routes(app: Router, state: &AppStateRef) -> Result<Router> {
    Ok(app.merge(routes::get_routes(state)))
}

async fn build_app() -> Result<Router> {
    // build state
    let app_state = Arc::new(AppState::new(
        build_database_connection().await?,
        build_auth_service()?,
    ));

    // build app
    let app = Router::new();
    let app = build_routes(app, &app_state)?;
    let app = build_service(app, &app_state)?;

    Ok(app)
}

async fn app_main() -> Result<()> {
    env_logger::init();

    let listen_addr = env::var("LCSM_LISTEN_ADDR")?;
    let listener = TcpListener::bind(listen_addr).await?;
    let app = build_app().await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = app_main().await {
        panic!("{}", e);
    }
}
