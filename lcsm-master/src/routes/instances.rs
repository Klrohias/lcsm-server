use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::Response,
    routing::get,
};
use reqwest::Method;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;

use crate::{
    AppStateRef, api_error,
    entities::slave,
    routes::slave_proxy::{find_slave, proxy_empty, proxy_json},
    services::auth::{self, Claims},
    trace_error,
};

pub fn get_routes(state: &AppStateRef) -> Router {
    let auth_middleware =
        middleware::from_fn_with_state(state.auth_service.clone(), auth::jwt_middleware);

    Router::new()
        .route("/", get(get_instances))
        .route("/{slave_id}", get(get_instances).put(create_instance))
        .route(
            "/{slave_id}/{instance_id}",
            get(get_instance)
                .patch(update_instance)
                .delete(delete_instance),
        )
        .route_layer(auth_middleware)
        .with_state(state.clone())
}

#[derive(Debug, Deserialize, Serialize)]
struct ProxyQuery {
    page: Option<u64>,
    page_size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstanceModel {
    id: i32,
    name: String,
    launch_command: String,
    work_dir: String,
    arguments: String,
    use_shell: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct PaginationResponse<T> {
    total: u64,
    page_count: u64,
    data: Vec<T>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstanceResponse {
    slave_id: i32,
    #[serde(flatten)]
    instance: InstanceModel,
}

async fn ensure_instance_permission(
    state: &AppStateRef,
    claims: &Claims,
    slave_id: i32,
    instance_id: i32,
) -> Result<slave::Model, Response> {
    state
        .permission_service
        .ensure_instance_permission(claims.id, slave_id, instance_id)
        .await
        .map_err(|status| api_error!(status))?;

    find_slave(state, slave_id).await
}

#[instrument(skip(state))]
async fn get_instances(
    State(state): State<AppStateRef>,
    slave_path: Option<Path<i32>>,
    Query(query): Query<ProxyQuery>,
) -> Result<Json<PaginationResponse<InstanceResponse>>, Response> {
    let slaves = if let Some(Path(slave_id)) = slave_path {
        vec![find_slave(&state, slave_id).await?]
    } else {
        slave::Entity::find()
            .all(&state.database_connection)
            .await
            .map_err(trace_error!(
                "find slaves",
                StatusCode::INTERNAL_SERVER_ERROR
            ))?
    };

    let mut data = Vec::new();
    let mut total = 0;
    for slave in slaves {
        let page: PaginationResponse<InstanceModel> =
            proxy_json(&slave, Method::GET, "/instance", Some(&query), None).await?;
        total += page.total;
        data.extend(page.data.into_iter().map(|instance| InstanceResponse {
            slave_id: slave.id,
            instance,
        }));
    }

    Ok(Json(PaginationResponse {
        total,
        page_count: 1,
        data,
    }))
}

#[instrument(skip(state))]
async fn get_instance(
    State(state): State<AppStateRef>,
    Extension(claims): Extension<Claims>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
) -> Result<Json<InstanceResponse>, Response> {
    let slave = ensure_instance_permission(&state, &claims, slave_id, instance_id).await?;
    let instance = proxy_json(
        &slave,
        Method::GET,
        &format!("/instance/{instance_id}"),
        None::<&ProxyQuery>,
        None,
    )
    .await?;

    Ok(Json(InstanceResponse { slave_id, instance }))
}

#[instrument(skip(state, payload))]
async fn create_instance(
    State(state): State<AppStateRef>,
    Path(slave_id): Path<i32>,
    Json(payload): Json<Value>,
) -> Result<Json<InstanceModel>, Response> {
    let slave = find_slave(&state, slave_id).await?;
    let instance = proxy_json(
        &slave,
        Method::PUT,
        "/instance",
        None::<&ProxyQuery>,
        Some(payload),
    )
    .await?;
    Ok(Json(instance))
}

#[instrument(skip(state, patch))]
async fn update_instance(
    State(state): State<AppStateRef>,
    Extension(claims): Extension<Claims>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
    Json(patch): Json<Value>,
) -> Result<Json<InstanceModel>, Response> {
    let slave = ensure_instance_permission(&state, &claims, slave_id, instance_id).await?;
    let instance = proxy_json(
        &slave,
        Method::PATCH,
        &format!("/instance/{instance_id}"),
        None::<&ProxyQuery>,
        Some(patch),
    )
    .await?;
    Ok(Json(instance))
}

#[instrument(skip(state))]
async fn delete_instance(
    State(state): State<AppStateRef>,
    Extension(claims): Extension<Claims>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
) -> Result<StatusCode, Response> {
    let slave = ensure_instance_permission(&state, &claims, slave_id, instance_id).await?;
    proxy_empty(
        &slave,
        Method::DELETE,
        &format!("/instance/{instance_id}"),
        None,
    )
    .await
}
