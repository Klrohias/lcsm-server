use axum::{
    Extension, Router,
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{Json, Response},
    routing::{delete, get, patch, post, put},
};
use bcrypt::{DEFAULT_COST, hash};
use futures::TryFutureExt;
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
    entities::user,
    services::{auth, permission_control},
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
    let create_user_middleware =
        middleware::from_fn_with_state(state.clone(), create_user_middleware);

    Router::new()
        // ---
        .route("/", post(create_user))
        .route_layer(
            ServiceBuilder::new()
                .layer(auth_middleware.clone())
                .layer(create_user_middleware),
        )
        // ---
        .route("/", get(get_users))
        .route("/:id", get(get_user))
        .route("/:id", delete(delete_user))
        .route("/:id", patch(update_user))
        .route("/:id/ban", put(ban_user))
        .route("/:id/ban", delete(unban_user))
        .route_layer(
            ServiceBuilder::new()
                .layer(auth_middleware.clone())
                .layer(admin_middleware.clone()),
        )
        // ---
        .route("/me", get(get_current_user))
        .route_layer(auth_middleware.clone())
        // ---
        .route("/login", post(login))
        .with_state(state.clone())
}

#[instrument(skip(state, request, next))]
async fn create_user_middleware(
    State(state): State<AppStateRef>,
    Extension(claims): Extension<auth::Claims>,
    request: Request,
    next: Next,
) -> Result<impl axum::response::IntoResponse, Response> {
    let db = &state.database_connection;

    // Check if any users exist
    let user_count = user::Entity::find().count(db).await.map_err(trace_error!(
        "check user exists",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    let is_admin = state.permission_service.is_administrator(claims.id).await;

    // Allow first user creation or admin users
    if user_count == 0 || is_admin {
        Ok(next.run(request).await)
    } else {
        Err(api_error!(StatusCode::FORBIDDEN))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub user_type: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub user_type: String,
}

impl From<user::Model> for UserResponse {
    fn from(model: user::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            email: model.email,
            user_type: model.user_type,
        }
    }
}

#[instrument(skip_all, fields(user.name = request.name, user.type = request.user_type))]
pub async fn create_user(
    State(state): State<AppStateRef>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, Response> {
    let db = &state.database_connection;

    // check if this is the first user
    let user_count = user::Entity::find().count(db).await.map_err(trace_error!(
        "check user exists",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    // the first user created must be administrator
    let user_type = if user_count == 0 {
        "administrator".to_string()
    } else {
        request.user_type.unwrap_or_else(|| "user".to_string())
    };

    // Hash the password
    let password_hash = hash(request.password, DEFAULT_COST).map_err(trace_error!(
        "hash password",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    let new_user = user::ActiveModel {
        name: Set(request.name),
        email: Set(request.email),
        password_hash: Set(password_hash),
        user_type: Set(user_type),
        ..Default::default()
    };

    let created_user = new_user.insert(db).await.map_err(trace_error!(
        "insert user",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    Ok(Json(UserResponse::from(created_user)))
}

#[instrument(skip(state))]
pub async fn get_user(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
) -> Result<Json<UserResponse>, Response> {
    match state.user_service.find_user_by_id(id).await {
        Ok(v) => Ok(Json(UserResponse::from(v))),
        Err(sea_orm::DbErr::RecordNotFound(_)) => return Err(api_error!(StatusCode::NOT_FOUND)),
        Err(e) => return Err(trace_error!("find user", StatusCode::NOT_FOUND)(e)),
    }
}

#[derive(Debug, Deserialize)]
pub struct UsersQuery {
    #[serde(rename = "id")]
    pub ids: Option<Vec<u64>>,
}

#[instrument(skip(state))]
pub async fn get_users(
    State(state): State<AppStateRef>,
    Query(pagination): Query<PaginationOptions>,
    Query(query): Query<UsersQuery>,
) -> Result<Json<PaginationResponse<UserResponse>>, Response> {
    let db = &state.database_connection;
    let page = pagination.page.unwrap_or(1);
    let page_size = pagination.page_size.unwrap_or(10);

    let mut paginator = user::Entity::find();
    if query.ids.is_some() {
        paginator = paginator.filter(user::Column::Id.is_in(query.ids.unwrap()));
    }

    let paginator = paginator.paginate(db, page_size);
    let num = paginator.num_items_and_pages().await.map_err(trace_error!(
        "num_items_and_pages",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    let models = paginator
        .fetch_page(page - 1)
        .await
        .map_err(trace_error!(
            "fetch_page",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?
        .into_iter()
        .map(|x| UserResponse::from(x))
        .collect();

    Ok(Json(PaginationResponse {
        page_count: num.number_of_pages,
        total: num.number_of_items,
        data: models,
    }))
}

#[instrument(skip(state))]
pub async fn delete_user(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
) -> Result<StatusCode, Response> {
    let user = match state.user_service.find_user_by_id(id).await {
        Ok(v) => v,
        Err(sea_orm::DbErr::RecordNotFound(_)) => return Err(api_error!(StatusCode::NOT_FOUND)),
        Err(e) => return Err(trace_error!("find user", StatusCode::NOT_FOUND)(e)),
    };

    user.delete(&state.database_connection)
        .await
        .map_err(trace_error!(
            "delete user",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

#[instrument(skip(state))]
pub async fn update_user(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
    Json(patch): Json<Patch>,
) -> Result<Json<UserResponse>, Response> {
    let db = &state.database_connection;

    let user = match state.user_service.find_user_by_id(id).await {
        Ok(v) => v,
        Err(sea_orm::DbErr::RecordNotFound(_)) => return Err(api_error!(StatusCode::NOT_FOUND)),
        Err(e) => return Err(trace_error!("find user", StatusCode::NOT_FOUND)(e)),
    };

    // Convert user to JSON for patch application
    let mut user_json = serde_json::to_value(&user).map_err(trace_error!(
        "to serde value",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    // Apply patch
    json_patch::patch(&mut user_json, &patch)
        .map_err(trace_error!("load json patch", StatusCode::BAD_REQUEST))?;

    // Convert back to user model
    let updated_user: user::Model = serde_json::from_value(user_json)
        .map_err(trace_error!("load patched model", StatusCode::BAD_REQUEST))?;

    let mut active_model = updated_user.into_active_model().reset_all();
    active_model.id = Unchanged(id);
    active_model.password_hash = Unchanged(user.password_hash);

    let updated_user = active_model.update(db).await.map_err(trace_error!(
        "update user",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    Ok(Json(UserResponse::from(updated_user)))
}

#[instrument(skip(state))]
pub async fn get_current_user(
    State(state): State<AppStateRef>,
    Extension(claims): Extension<auth::Claims>,
) -> Result<Json<UserResponse>, Response> {
    match state.user_service.find_user_by_id(claims.id).await {
        Ok(v) => Ok(Json(UserResponse::from(v))),
        Err(sea_orm::DbErr::RecordNotFound(_)) => return Err(api_error!(StatusCode::NOT_FOUND)),
        Err(e) => return Err(trace_error!("find user", StatusCode::NOT_FOUND)(e)),
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub access_token: String,
}

#[instrument(skip_all, fields(user.email = request.username))]
pub async fn login(
    State(state): State<AppStateRef>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, Response> {
    // verify the user
    let user = state
        .user_service
        .verify_user_creds(request.username, request.password)
        .map_err(trace_error!("verify user creds", StatusCode::UNAUTHORIZED))
        .await?;

    // generate jwt token
    let token = state
        .auth_service
        .create_jwt(user.id, &user.email)
        .map_err(trace_error!(
            "create_jwt",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?;

    Ok(Json(LoginResponse {
        access_token: token,
    }))
}

#[instrument(skip(state))]
pub async fn ban_user(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
    Extension(claims): Extension<auth::Claims>,
) -> Result<(), Response> {
    // cannot ban self
    if id == claims.id {
        return Err(api_error!(
            "you can't ban yourself".to_string(),
            StatusCode::NOT_ACCEPTABLE
        ));
    }

    // set ban
    state
        .user_service
        .set_user_banned(id, true)
        .await
        .map_err(trace_error!(
            "set user banned",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?;

    Ok(())
}

#[instrument(skip(state))]
pub async fn unban_user(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
) -> Result<(), Response> {
    // set ban
    state
        .user_service
        .set_user_banned(id, false)
        .await
        .map_err(trace_error!(
            "set user banned",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?;

    Ok(())
}
