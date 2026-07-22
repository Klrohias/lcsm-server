use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::Response,
    routing::{get, put},
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    AppStateRef, api_error,
    routes::slave_proxy::{find_slave, proxy_empty, proxy_text, slave_url},
    services::{auth, permission_control},
    trace_error,
};

pub fn get_routes(state: &AppStateRef) -> Router {
    let auth_middleware =
        middleware::from_fn_with_state(state.auth_service.clone(), auth::jwt_middleware);
    let admin_middleware = middleware::from_fn_with_state(
        state.permission_service.clone(),
        permission_control::admin_middleware,
    );

    Router::new()
        .route(
            "/{slave_id}/{instance_id}",
            put(start_process).delete(kill_process).get(process_state),
        )
        .route("/{slave_id}/{instance_id}/logs", get(fetch_process_log))
        .route_layer(
            tower::ServiceBuilder::new()
                .layer(auth_middleware)
                .layer(admin_middleware),
        )
        .with_state(state.clone())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessStateResponse {
    running: bool,
}

#[instrument(skip(state))]
async fn start_process(
    State(state): State<AppStateRef>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
) -> Result<StatusCode, Response> {
    let slave = find_slave(&state, slave_id).await?;
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
    Path((slave_id, instance_id)): Path<(i32, i32)>,
) -> Result<StatusCode, Response> {
    let slave = find_slave(&state, slave_id).await?;
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
    Path((slave_id, instance_id)): Path<(i32, i32)>,
) -> Result<Json<ProcessStateResponse>, Response> {
    let slave = find_slave(&state, slave_id).await?;
    let response = reqwest::Client::new()
        .get(slave_url(&slave, &format!("/process/{instance_id}")))
        .bearer_auth(&slave.slave_token)
        .send()
        .await
        .map_err(trace_error!("request slave", StatusCode::BAD_GATEWAY))?;

    match response.status() {
        reqwest::StatusCode::OK => Ok(Json(ProcessStateResponse { running: true })),
        reqwest::StatusCode::NOT_FOUND => Ok(Json(ProcessStateResponse { running: false })),
        status => Err(api_error!(
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
        )),
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct LogQuery {
    tail: Option<usize>,
}

#[instrument(skip(state))]
async fn fetch_process_log(
    State(state): State<AppStateRef>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
    Query(query): Query<LogQuery>,
) -> Result<String, Response> {
    let slave = find_slave(&state, slave_id).await?;
    let text = proxy_text(
        &slave,
        Method::GET,
        &format!("/process/{instance_id}/logs"),
        Option::<&()>::None,
    )
    .await?;

    if let Some(tail) = query.tail {
        let mut lines = text.lines().rev().take(tail).collect::<Vec<_>>();
        lines.reverse();
        Ok(lines.join("\n"))
    } else {
        Ok(text)
    }
}
