use std::{env, process::ExitCode, sync::Arc};

use axum::Router;
use bcrypt::{DEFAULT_COST, hash};
use lcsm_master::{
    AppState, AppStateRef,
    database::build_database_connection,
    entities::user,
    routes, schema,
    services::{AuthService, AuthServiceRef, PermissionService, UserService},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use typed_container::Container;

const SERVICE_NAME: &str = "master";

fn build_auth_service() -> AuthServiceRef {
    AuthService::new(env::var("LCSM_JWT_SECRET").unwrap_or_else(|_| "lcsm-dev-secret".to_owned()))
}

fn build_service(app: Router) -> Router {
    app.layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
}

fn build_routes(app: Router, state: &AppStateRef) -> Router {
    app.merge(routes::get_routes(state))
}

async fn build_app() -> Router {
    // build state
    let c = Container::new();
    c.register_service(build_database_connection(SERVICE_NAME).await);
    c.register_service(build_auth_service());
    c.register_constructor(|c| Arc::new(PermissionService::new(c.get())));
    c.register_constructor(|c| Arc::new(UserService::new(c.get())));
    c.register_constructor(|c| Arc::new(AppState::from(c)));

    let app_state = c.get();
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

async fn serve() -> anyhow::Result<()> {
    let listen_addr = env::var("LCSM_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
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

async fn add_admin(name: String, password: String) -> anyhow::Result<()> {
    let db = build_database_connection(SERVICE_NAME).await;
    schema::sync_schema(&db).await?;

    let password_hash = hash(password, DEFAULT_COST)?;
    let email = name.clone();
    let existing = user::Entity::find()
        .filter(user::Column::Name.eq(&name))
        .one(&db)
        .await?;

    match existing {
        Some(existing) => {
            let active = user::ActiveModel {
                id: Set(existing.id),
                name: Set(name),
                email: Set(existing.email),
                password_hash: Set(password_hash),
                user_type: Set("administrator".to_owned()),
                banned: Set(false),
            };
            active.update(&db).await?;
            println!("administrator updated");
        }
        None => {
            let active = user::ActiveModel {
                name: Set(name),
                email: Set(email),
                password_hash: Set(password_hash),
                user_type: Set("administrator".to_owned()),
                banned: Set(false),
                ..Default::default()
            };
            active.insert(&db).await?;
            println!("administrator created");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();

    let result = match env::args().collect::<Vec<_>>().as_slice() {
        [_, command] if command == "internal-sync-entity" => sync_entity().await,
        [_, command, name, password] if command == "addadmin" => {
            add_admin(name.clone(), password.clone()).await
        }
        [_, command] if command == "addadmin" => {
            Err(anyhow::anyhow!("usage: lcsm-master addadmin <user> <pass>"))
        }
        [_, command, ..] => Err(anyhow::anyhow!("unknown command: {command}")),
        [_] => serve().await,
        [] => serve().await,
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "lcsm-master failed");
            eprintln!("lcsm-master failed: {error}");
            ExitCode::FAILURE
        }
    }
}
