use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::Response,
    routing::get,
};
use reqwest::Method;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use tracing::instrument;

use crate::{
    AppStateRef, api_error,
    entities::slave,
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
        .route("/", get(get_instances))
        .route("/{slave_id}", get(get_instances).put(create_instance))
        .route(
            "/{slave_id}/{instance_id}",
            get(get_instance)
                .patch(update_instance)
                .delete(delete_instance),
        )
        .route_layer(
            tower::ServiceBuilder::new()
                .layer(auth_middleware)
                .layer(admin_middleware),
        )
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

async fn find_slave(state: &AppStateRef, id: i32) -> Result<slave::Model, Response> {
    slave::Entity::find_by_id(id)
        .one(&state.database_connection)
        .await
        .map_err(trace_error!(
            "find slave",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?
        .ok_or(api_error!(StatusCode::NOT_FOUND))
}

fn slave_url(slave: &slave::Model, path: &str) -> String {
    format!(
        "{}/{}",
        slave.slave_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

async fn proxy_json<T: DeserializeOwned>(
    slave: &slave::Model,
    method: Method,
    path: &str,
    query: Option<&ProxyQuery>,
    body: Option<Value>,
) -> Result<T, Response> {
    let client = reqwest::Client::new();
    let mut request = client
        .request(method, slave_url(slave, path))
        .bearer_auth(&slave.slave_token);

    if let Some(query) = query {
        request = request.query(query);
    }
    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = request
        .send()
        .await
        .map_err(trace_error!("request slave", StatusCode::BAD_GATEWAY))?;
    let status = response.status();
    if !status.is_success() {
        return Err(api_error!(
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
        ));
    }

    response.json::<T>().await.map_err(trace_error!(
        "decode slave response",
        StatusCode::BAD_GATEWAY
    ))
}

async fn proxy_empty(
    slave: &slave::Model,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<StatusCode, Response> {
    let client = reqwest::Client::new();
    let mut request = client
        .request(method, slave_url(slave, path))
        .bearer_auth(&slave.slave_token);
    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = request
        .send()
        .await
        .map_err(trace_error!("request slave", StatusCode::BAD_GATEWAY))?;
    let status = response.status();
    if !status.is_success() {
        return Err(api_error!(
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
        ));
    }

    Ok(StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::OK))
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
    Path((slave_id, instance_id)): Path<(i32, i32)>,
) -> Result<Json<InstanceResponse>, Response> {
    let slave = find_slave(&state, slave_id).await?;
    let instance = proxy_json(
        &slave,
        Method::GET,
        &format!("/instance/{instance_id}"),
        None,
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
    let instance = proxy_json(&slave, Method::PUT, "/instance", None, Some(payload)).await?;
    Ok(Json(instance))
}

#[instrument(skip(state, patch))]
async fn update_instance(
    State(state): State<AppStateRef>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
    Json(patch): Json<Value>,
) -> Result<Json<InstanceModel>, Response> {
    let slave = find_slave(&state, slave_id).await?;
    let instance = proxy_json(
        &slave,
        Method::PATCH,
        &format!("/instance/{instance_id}"),
        None,
        Some(patch),
    )
    .await?;
    Ok(Json(instance))
}

#[instrument(skip(state))]
async fn delete_instance(
    State(state): State<AppStateRef>,
    Path((slave_id, instance_id)): Path<(i32, i32)>,
) -> Result<StatusCode, Response> {
    let slave = find_slave(&state, slave_id).await?;
    proxy_empty(
        &slave,
        Method::DELETE,
        &format!("/instance/{instance_id}"),
        None,
    )
    .await
}
