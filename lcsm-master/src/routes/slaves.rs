use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    routing::get,
};
use json_patch::Patch;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Unchanged, ColumnTrait, EntityTrait, IntoActiveModel,
    ModelTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::Deserialize;
use tower::ServiceBuilder;
use tracing::instrument;

use crate::{
    AppStateRef,
    entities::slave,
    services::auth,
    trace_error,
    transfer::{PaginationOptions, PaginationResponse},
};

pub fn get_routes(state: &AppStateRef) -> Router {
    Router::new()
        .route("/", get(get_slaves).post(create_slave))
        .route(
            "/:id",
            get(get_slave).delete(delete_slave).patch(update_slave),
        )
        .route_layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn_with_state(
                    state.auth_service.clone(),
                    auth::jwt_middleware,
                ))
                .layer(middleware::from_fn(auth::admin_middleware)),
        )
        .with_state(state.clone())
}

#[derive(Debug, Deserialize)]
pub struct CreateSlaveRequest {
    pub name: String,
    pub description: String,
    pub slave_url: String,
    pub slave_token: String,
}

#[instrument(skip(state))]
pub async fn create_slave(
    State(state): State<AppStateRef>,
    Json(request): Json<CreateSlaveRequest>,
) -> Result<Json<slave::Model>, StatusCode> {
    let db = &state.database_connection;

    let new_slave = slave::ActiveModel {
        name: Set(request.name),
        description: Set(request.description),
        slave_token: Set(request.slave_token),
        slave_url: Set(request.slave_url),
        ..Default::default()
    };

    let created_slave = new_slave.insert(db).await.map_err(trace_error!(
        "insert slave",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    Ok(Json(created_slave))
}

#[instrument(skip(state))]
pub async fn get_slave(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
) -> Result<Json<slave::Model>, StatusCode> {
    let db = &state.database_connection;

    let slave = slave::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(trace_error!(
            "find slave",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(slave))
}

#[derive(Debug, Deserialize)]
pub struct SlavesQuery {
    #[serde(rename = "id")]
    pub ids: Option<Vec<u64>>,
}

#[instrument(skip(state))]
pub async fn get_slaves(
    State(state): State<AppStateRef>,
    Query(pagination): Query<PaginationOptions>,
    Query(query): Query<SlavesQuery>,
) -> Result<Json<PaginationResponse<slave::Model>>, StatusCode> {
    let db = &state.database_connection;
    let page = pagination.page.unwrap_or(1);
    let page_size = pagination.page_size.unwrap_or(10);

    let mut paginator = slave::Entity::find();
    if query.ids.is_some() {
        paginator = paginator.filter(slave::Column::Id.is_in(query.ids.unwrap()));
    }

    let paginator = paginator.paginate(db, page_size);
    let num = paginator.num_items_and_pages().await.map_err(trace_error!(
        "num_items_and_pages",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    let models = paginator.fetch_page(page - 1).await.map_err(trace_error!(
        "fetch_page",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    Ok(Json(PaginationResponse {
        page_count: num.number_of_pages,
        total: num.number_of_items,
        data: models,
    }))
}

#[instrument(skip(state))]
pub async fn delete_slave(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let db = &state.database_connection;

    let slave = slave::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(trace_error!(
            "find slave",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?
        .ok_or(StatusCode::NOT_FOUND)?;

    slave.delete(db).await.map_err(trace_error!(
        "delete slave",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    Ok(StatusCode::NO_CONTENT)
}

#[instrument(skip(state, patch))]
pub async fn update_slave(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
    Json(patch): Json<Patch>,
) -> Result<Json<slave::Model>, StatusCode> {
    let db = &state.database_connection;

    let slave = slave::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(trace_error!(
            "find slave",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut slave_json = serde_json::to_value(&slave).map_err(trace_error!(
        "to serde value",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    json_patch::patch(&mut slave_json, &patch)
        .map_err(trace_error!("load json patch", StatusCode::BAD_REQUEST))?;

    let updated_slave: slave::Model = serde_json::from_value(slave_json)
        .map_err(trace_error!("load patched model", StatusCode::BAD_REQUEST))?;

    let mut active_model = updated_slave.into_active_model().reset_all();
    active_model.id = Unchanged(id);

    let updated_slave = active_model.update(db).await.map_err(trace_error!(
        "update slave",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    Ok(Json(updated_slave))
}
