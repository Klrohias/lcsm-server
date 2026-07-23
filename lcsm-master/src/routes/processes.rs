use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{any, get, put},
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    AppStateRef, api_error,
    routes::slave_proxy::{find_slave, proxy_empty, proxy_json, proxy_websocket},
    services::auth::{self, Claims},
};

pub fn get_routes(state: &AppStateRef) -> Router {
    let auth_middleware = middleware::from_fn_with_state(
        state.auth_service.clone(),
        auth::jwt_middleware,
    );
    let ws_auth_middleware = middleware::from_fn_with_state(
        state.auth_service.clone(),
        auth::jwt_header_or_query_middleware,
    );

    Router::new()
        .route(
            "/{slave_id}/{instance_id}",
            put(start_process).delete(kill_process).get(process_state),
        )
        .route("/{slave_id}/{instance_id}/logs", get(fetch_process_log))
        .route_layer(auth_middleware)
        .route("/{slave_id}/{instance_id}/terminal", any(terminal_ws_connect))
        .route_layer(ws_auth_middleware)
        .with_state(state.clone())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessStateResponse {
    status: String,
    is_running: bool,
    pid: Option<u32>,
    started_at: Option<u64>,
    uptime_ms: Option<u64>,
    cpu_percent: Option<f32>,
    memory_bytes: Option<u64>,
}

async fn ensure_instance_permission(
    state: &AppStateRef,
    claims: &Claims,
    slave_id: i32,
    instance_id: i32,
) -> Result<crate::entities::slave::Model, Response> {
    state
        .permission_service
        .ensure_instance_permission(claims.id, slave_id, instance_id)
        .await
        .map_err(|status| api_error!(status))?;

    find_slave(state, slave_id).await
}

#[instrument(skip(state))]
async fn start_process(
    State(state): State<AppStateRef>,
    Extension(claims): Extension<Claims>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
) -> Result<StatusCode, Response> {
    let slave = ensure_instance_permission(&state, &claims, slave_id, instance_id).await?;
    proxy_empty(
        &slave,
        Method::PUT,
        &format!("/process/{instance_id}"),
        None,
    )
    .await
}

#[instrument(skip(state))]
async fn kill_process(
    State(state): State<AppStateRef>,
    Extension(claims): Extension<Claims>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
) -> Result<StatusCode, Response> {
    let slave = ensure_instance_permission(&state, &claims, slave_id, instance_id).await?;
    proxy_empty(
        &slave,
        Method::DELETE,
        &format!("/process/{instance_id}"),
        None,
    )
    .await
}

#[instrument(skip(state))]
async fn process_state(
    State(state): State<AppStateRef>,
    Extension(claims): Extension<Claims>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
) -> Result<Json<ProcessStateResponse>, Response> {
    let slave = ensure_instance_permission(&state, &claims, slave_id, instance_id).await?;
    let runtime = proxy_json(
        &slave,
        Method::GET,
        &format!("/process/{instance_id}"),
        None::<&()>,
        None,
    )
    .await?;
    Ok(Json(runtime))
}

#[derive(Debug, Deserialize, Serialize)]
struct LogQuery {
    before: Option<u64>,
    limit: Option<usize>,
    tail: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogChunkResponse {
    content: String,
    has_more: bool,
    begin_offset: u64,
}

#[instrument(skip(state))]
async fn fetch_process_log(
    State(state): State<AppStateRef>,
    Extension(claims): Extension<Claims>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
    Query(query): Query<LogQuery>,
) -> Result<Json<LogChunkResponse>, Response> {
    let slave = ensure_instance_permission(&state, &claims, slave_id, instance_id).await?;
    let chunk: LogChunkResponse = proxy_json(
        &slave,
        Method::GET,
        &format!("/process/{instance_id}/logs"),
        Some(&query),
        None,
    )
    .await?;
    Ok(Json(chunk))
}

#[instrument(skip(state, ws))]
async fn terminal_ws_connect(
    State(state): State<AppStateRef>,
    Extension(claims): Extension<Claims>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, Response> {
    let slave = ensure_instance_permission(&state, &claims, slave_id, instance_id).await?;
    let path = format!("/process/{instance_id}/terminal");

    Ok(ws.on_upgrade(move |socket| proxy_websocket(slave, path, socket)))
}
