use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::Response,
    routing::{delete, get, post},
};
use json_patch::Patch;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Unchanged, ColumnTrait, EntityTrait, IntoActiveModel,
    ModelTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tracing::instrument;

use crate::{
    AppStateRef, api_error,
    entities::slave,
    services::{
        auth::{self, Claims},
        permission_control,
    },
    trace_error,
    transfer::{PaginationOptions, PaginationResponse},
};

pub fn get_routes(state: &AppStateRef) -> Router {
    let auth_middleware =
        middleware::from_fn_with_state(state.auth_service.clone(), auth::jwt_middleware);
    let admin_middleware = middleware::from_fn_with_state(
        state.permission_service.clone(),
        permission_control::admin_middleware,
    );

    Router::new()
        // ---
        .route("/", get(get_slaves))
        .route("/:id", get(get_slave))
        .route_layer(middleware::from_fn_with_state(
            state.auth_service.clone(),
            auth::jwt_middleware,
        ))
        // ---
        .route("/", post(create_slave))
        .route("/:id", delete(delete_slave).patch(update_slave))
        .route_layer(
            ServiceBuilder::new()
                .layer(auth_middleware.clone())
                .layer(admin_middleware.clone()),
        )
        .with_state(state.clone())
}

#[derive(Serialize)]
pub struct BirefSlave {
    pub id: i32,
    pub name: String,
    pub description: String,
}

impl From<slave::Model> for BirefSlave {
    fn from(value: slave::Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum SlaveResponse {
    Detailed(slave::Model),
    Biref(BirefSlave),
}

impl SlaveResponse {
    fn into_biref(self) -> Self {
        match self {
            Self::Detailed(v) => Self::Biref(v.into()),
            Self::Biref(v) => Self::Biref(v),
        }
    }
}

impl From<slave::Model> for SlaveResponse {
    fn from(value: slave::Model) -> Self {
        Self::Detailed(value)
    }
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
) -> Result<Json<slave::Model>, Response> {
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
    Extension(claims): Extension<Claims>,
) -> Result<Json<SlaveResponse>, Response> {
    let db = &state.database_connection;

    let is_admin = state.permission_service.is_administrator(claims.id).await;

    let slave = slave::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(trace_error!(
            "find slave",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?
        .ok_or(api_error!(StatusCode::NOT_FOUND))?;

    let slave = if is_admin {
        slave.into()
    } else {
        SlaveResponse::from(slave).into_biref()
    };

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
    Extension(claims): Extension<Claims>,
) -> Result<Json<PaginationResponse<SlaveResponse>>, Response> {
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

    let is_admin = state.permission_service.is_administrator(claims.id).await;

    let models = paginator
        .fetch_page(page - 1)
        .await
        .map_err(trace_error!(
            "fetch_page",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?
        .into_iter()
        .map(|x| {
            let slave = SlaveResponse::from(x);
            if is_admin { slave } else { slave.into_biref() }
        })
        .collect();

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
) -> Result<StatusCode, Response> {
    let db = &state.database_connection;

    let slave = slave::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(trace_error!(
            "find slave",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?
        .ok_or(api_error!(StatusCode::NOT_FOUND))?;

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
) -> Result<Json<slave::Model>, Response> {
    let db = &state.database_connection;

    let slave = slave::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(trace_error!(
            "find slave",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?
        .ok_or(api_error!(StatusCode::NOT_FOUND))?;

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
