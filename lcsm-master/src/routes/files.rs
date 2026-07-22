use axum::{
    Json, Router,
    extract::{Path, Query, State},
    middleware,
    response::Response,
    routing::get,
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    AppStateRef,
    routes::slave_proxy::{find_slave, proxy_json},
    services::{auth, permission_control},
};

pub fn get_routes(state: &AppStateRef) -> Router {
    let auth_middleware =
        middleware::from_fn_with_state(state.auth_service.clone(), auth::jwt_middleware);
    let admin_middleware = middleware::from_fn_with_state(
        state.permission_service.clone(),
        permission_control::admin_middleware,
    );

    Router::new()
        .route("/{slave_id}/{instance_id}", get(list_files))
        .route("/{slave_id}/{instance_id}/tree", get(list_files))
        .route_layer(
            tower::ServiceBuilder::new()
                .layer(auth_middleware)
                .layer(admin_middleware),
        )
        .with_state(state.clone())
}

#[derive(Debug, Deserialize, Serialize)]
struct FileQuery {
    path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    modified: Option<u64>,
}

#[instrument(skip(state))]
async fn list_files(
    State(state): State<AppStateRef>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
    Query(query): Query<FileQuery>,
) -> Result<Json<Vec<FileEntry>>, Response> {
    let slave = find_slave(&state, slave_id).await?;
    let entries = proxy_json(
        &slave,
        Method::GET,
        &format!("/file/{instance_id}"),
        Some(&query),
        None,
    )
    .await?;
    Ok(Json(entries))
}
