use std::{env, process::ExitCode, sync::Arc};

use axum::Router;
use lcsm_slave::{
    AppState, AppStateRef,
    database::{build_database_connection, get_data_path},
    routes, schema,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, validate_request::ValidateRequestHeaderLayer};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const SERVICE_NAME: &str = "slave";

fn build_services(app: Router) -> Router {
    let token = env::var("LCSM_SLAVE_TOKEN").unwrap_or_else(|_| "lcsm-slave-dev-token".to_owned());

    app.layer(
        ServiceBuilder::new()
            .layer(CorsLayer::permissive())
            .layer(ValidateRequestHeaderLayer::bearer(&token)),
    )
}

fn build_routes(app: Router, state: &AppStateRef) -> Router {
    app.merge(routes::get_routes(state))
}

async fn build_app() -> Router {
    // build state
    let app_state = Arc::new(AppState::new(
        build_database_connection(SERVICE_NAME).await,
        get_data_path(SERVICE_NAME),
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

async fn serve() -> anyhow::Result<()> {
    let listen_addr = env::var("LCSM_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:3001".to_owned());
    let listener = TcpListener::bind(listen_addr).await?;
    let app = build_app().await;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn sync_entity() -> anyhow::Result<()> {
    let db = build_database_connection(SERVICE_NAME).await;
    schema::sync_schema(&db).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();

    let result = match env::args().nth(1).as_deref() {
        Some("internal-sync-entity") => sync_entity().await,
        Some(command) => Err(anyhow::anyhow!("unknown command: {command}")),
        None => serve().await,
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "lcsm-slave failed");
            eprintln!("lcsm-slave failed: {error}");
            ExitCode::FAILURE
        }
    }
}
