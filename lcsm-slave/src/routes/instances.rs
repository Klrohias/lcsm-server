use crate::{
    AppStateRef,
    entities::instance,
    errors::trace_error,
    transfer::{PaginationOptions, PaginationResponse},
};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use axum_extra::extract::Query as ExtraQuery;
use json_patch::{PatchOperation, patch as apply_json_patch};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use serde::Deserialize;
use serde_json::Value;
use tracing::instrument;

pub fn get_routes(state_ref: &AppStateRef) -> Router {
    Router::new()
        .route("/", get(get_instances).put(create_instance))
        .route(
            "/{id}",
            get(get_instance)
                .patch(update_instance)
                .delete(delete_instance),
        )
        .with_state(state_ref.clone())
}

#[derive(Debug, Deserialize)]
struct InstancesQuery {
    #[serde(rename = "id")]
    pub ids: Option<Vec<i32>>,
}

#[instrument(skip(state))]
async fn get_instances(
    State(state): State<AppStateRef>,
    Query(pagination): Query<PaginationOptions>,
    ExtraQuery(query): ExtraQuery<InstancesQuery>,
) -> Result<Json<PaginationResponse<instance::Model>>, StatusCode> {
    let db = &state.database;
    let (page, page_size) = pagination.normalized();

    let mut paginator = instance::Entity::find().order_by_asc(instance::Column::Id);
    if let Some(ids) = query.ids {
        paginator = paginator.filter(instance::Column::Id.is_in(ids));
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
async fn get_instance(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
) -> Result<Json<instance::Model>, StatusCode> {
    let db = &state.database;
    let it = instance::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(trace_error!("one from db", StatusCode::BAD_REQUEST))?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(it))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateInstanceRequest {
    pub name: String,
    pub launch_command: String,
    #[serde(default)]
    pub work_dir: Option<String>,
    pub arguments: String,
    pub use_shell: bool,
}

#[instrument(skip(state))]
async fn create_instance(
    State(state): State<AppStateRef>,
    Json(payload): Json<CreateInstanceRequest>,
) -> Result<Json<instance::Model>, StatusCode> {
    let db = &state.database;
    let active = instance::ActiveModel {
        name: Set(payload.name),
        launch_command: Set(payload.launch_command),
        work_dir: Set(payload.work_dir.unwrap_or_default()),
        arguments: Set(payload.arguments),
        use_shell: Set(payload.use_shell),
        ..Default::default()
    };

    let res = active
        .insert(db)
        .await
        .map_err(trace_error!("insert", StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Json(res))
}

#[instrument(skip(state))]
async fn update_instance(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
    Json(patch_ops): Json<Value>, // 传入 JSON Patch 格式
) -> Result<Json<instance::Model>, StatusCode> {
    let db = &state.database;
    let model = instance::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(trace_error!(
            "one from db",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?
        .ok_or(StatusCode::NOT_FOUND)?;

    // preapre
    let mut value = serde_json::to_value(&model).map_err(trace_error!(
        "to serde value",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;
    let patch_ops_vec: Vec<PatchOperation> = serde_json::from_value(patch_ops)
        .map_err(trace_error!("parse json patch", StatusCode::BAD_REQUEST))?;

    // apply
    apply_json_patch(&mut value, &patch_ops_vec)
        .map_err(trace_error!("apply_json_patch", StatusCode::BAD_REQUEST))?;

    // check
    let updated: instance::Model = serde_json::from_value(value)
        .map_err(trace_error!("get new model", StatusCode::BAD_REQUEST))?;

    let updated = instance::ActiveModel {
        id: Set(id),
        name: Set(updated.name),
        launch_command: Set(updated.launch_command),
        work_dir: Set(updated.work_dir),
        arguments: Set(updated.arguments),
        use_shell: Set(updated.use_shell),
    };

    let res = updated.update(db).await.map_err(trace_error!(
        "update to db",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;
    Ok(Json(res))
}

#[instrument(skip(state))]
async fn delete_instance(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let db = &state.database;
    let res = instance::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(trace_error!(
            "exec delete",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?;
    if res.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
