use crate::{
    AppStateRef,
    entities::instance,
    errors::{bad_request_with_log, internal_error_with_log},
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
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter,
};
use serde::Deserialize;
use serde_json::Value;

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

#[derive(Deserialize)]
struct InstancesQuery {
    pub ids: Option<Vec<u64>>,
}

async fn get_instances(
    State(state): State<AppStateRef>,
    Query(pagination): Query<PaginationOptions>,
    ExtraQuery(query): ExtraQuery<InstancesQuery>,
) -> Result<Json<PaginationResponse<instance::Model>>, StatusCode> {
    let db = &state.db;
    let page = pagination.page.unwrap_or(1);
    let page_size = pagination.page_size.unwrap_or(10);

    let mut paginator = instance::Entity::find();
    if query.ids.is_some() {
        paginator = paginator.filter(instance::Column::Id.is_in(query.ids.unwrap()));
    }

    let paginator = paginator.paginate(db, page_size);
    let num = paginator
        .num_items_and_pages()
        .await
        .map_err(internal_error_with_log())?;

    let models = paginator
        .fetch_page(page - 1)
        .await
        .map_err(internal_error_with_log())?;

    Ok(Json(PaginationResponse {
        page_count: num.number_of_pages,
        total: num.number_of_items,
        data: models,
    }))
}

async fn get_instance(
    State(state): State<AppStateRef>,
    Path(id): Path<u64>,
) -> Result<Json<instance::Model>, StatusCode> {
    let db = &state.db;
    let it =
        instance::Entity::find_by_id(TryInto::<i32>::try_into(id).map_err(bad_request_with_log())?)
            .one(db)
            .await
            .map_err(internal_error_with_log())?
            .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(it))
}

async fn create_instance(
    State(state): State<AppStateRef>,
    Json(payload): Json<instance::Model>,
) -> Result<Json<instance::Model>, StatusCode> {
    let db = &state.db;
    let active = instance::ActiveModel {
        id: NotSet, // empty the id
        ..payload.into()
    };

    let res = active.insert(db).await.map_err(internal_error_with_log())?;
    Ok(Json(res))
}

async fn update_instance(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
    Json(patch_ops): Json<Value>, // 传入 JSON Patch 格式
) -> Result<Json<instance::Model>, StatusCode> {
    let db = &state.db;
    let model = instance::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(internal_error_with_log())?
        .ok_or(StatusCode::NOT_FOUND)?;

    // preapre
    let mut value = serde_json::to_value(&model).map_err(internal_error_with_log())?;
    let patch_ops_vec: Vec<PatchOperation> =
        serde_json::from_value(patch_ops).map_err(bad_request_with_log())?;

    // apply
    apply_json_patch(&mut value, &patch_ops_vec).map_err(bad_request_with_log())?;

    // check
    let updated: instance::Model = serde_json::from_value(value).map_err(bad_request_with_log())?;

    if updated.id != model.id {
        // avoid changing the id
        return Err(StatusCode::NOT_ACCEPTABLE);
    }

    let updated = updated.into_active_model().reset_all();
    let res = updated
        .update(db)
        .await
        .map_err(internal_error_with_log())?;
    Ok(Json(res))
}

async fn delete_instance(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let db = &state.db;
    let res = instance::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(internal_error_with_log())?;
    if res.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
