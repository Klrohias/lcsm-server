use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    middleware,
    response::Response,
    routing::get,
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    AppStateRef, api_error,
    routes::slave_proxy::{find_slave, proxy_json},
    services::auth::{self, Claims},
};

pub fn get_routes(state: &AppStateRef) -> Router {
    let auth_middleware =
        middleware::from_fn_with_state(state.auth_service.clone(), auth::jwt_middleware);

    Router::new()
        .route("/{slave_id}/{instance_id}", get(list_files))
        .route("/{slave_id}/{instance_id}/tree", get(list_files))
        .route_layer(auth_middleware)
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
    Extension(claims): Extension<Claims>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
    Query(query): Query<FileQuery>,
) -> Result<Json<Vec<FileEntry>>, Response> {
    state
        .permission_service
        .ensure_instance_permission(claims.id, slave_id, instance_id)
        .await
        .map_err(|status| api_error!(status))?;

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
