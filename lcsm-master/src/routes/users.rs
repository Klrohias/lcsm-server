use axum::{
    Router,
    extract::{Extension, Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Json,
    routing::{delete, get, patch, post},
};
use bcrypt::{DEFAULT_COST, hash};
use json_patch::Patch;
use sea_orm::{ActiveModelTrait, IntoActiveModel, Set, entity::prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    AppStateRef, bad_request_with_log, entities::user, internal_error_with_log, services::Claims,
};

async fn admin_middleware(
    Extension(claims): Extension<Claims>,
    request: Request,
    next: Next,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    if claims.user_type != "administrator" {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(request).await)
}

async fn create_user_middleware(
    State(state): State<AppStateRef>,
    Extension(claims): Extension<Claims>,
    request: Request,
    next: Next,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let db = &state.database_connection;

    // Check if any users exist
    let user_count = user::Entity::find()
        .count(db)
        .await
        .map_err(internal_error_with_log!())?;

    // Allow first user creation or admin users
    if user_count == 0 || claims.user_type == "administrator" {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

#[derive(Serialize, Deserialize)]
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

pub async fn create_user(
    State(state): State<AppStateRef>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    let db = &state.database_connection;

    // check if this is the first user
    let user_count = user::Entity::find()
        .count(db)
        .await
        .map_err(internal_error_with_log!())?;

    // the first user created must be administrator
    let user_type = if user_count == 0 {
        "administrator".to_string()
    } else {
        request.user_type.unwrap_or_else(|| "user".to_string())
    };

    // Hash the password
    let password_hash = hash(request.password, DEFAULT_COST).map_err(internal_error_with_log!())?;

    let new_user = user::ActiveModel {
        name: Set(request.name),
        email: Set(request.email),
        password_hash: Set(password_hash),
        user_type: Set(user_type),
        ..Default::default()
    };

    let created_user = new_user
        .insert(db)
        .await
        .map_err(internal_error_with_log!())?;

    Ok(Json(UserResponse::from(created_user)))
}

pub async fn get_user(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
) -> Result<Json<UserResponse>, StatusCode> {
    let db = &state.database_connection;

    let user = user::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(internal_error_with_log!())?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn get_users(
    State(state): State<AppStateRef>,
) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    let db = &state.database_connection;

    let users = user::Entity::find()
        .all(db)
        .await
        .map_err(internal_error_with_log!())?;

    let response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(response))
}

pub async fn delete_user(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let db = &state.database_connection;

    let user = user::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(internal_error_with_log!())?
        .ok_or(StatusCode::NOT_FOUND)?;

    user.delete(db).await.map_err(internal_error_with_log!())?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_user(
    State(state): State<AppStateRef>,
    Path(id): Path<i32>,
    Json(patch): Json<Patch>,
) -> Result<Json<UserResponse>, StatusCode> {
    let db = &state.database_connection;

    let user = user::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(internal_error_with_log!())?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Convert user to JSON for patch application
    let mut user_json = serde_json::to_value(&user).map_err(internal_error_with_log!())?;

    // Apply patch
    json_patch::patch(&mut user_json, &patch).map_err(bad_request_with_log!())?;

    // Convert back to user model
    let updated_user: user::Model =
        serde_json::from_value(user_json).map_err(bad_request_with_log!())?;

    let mut active_model = updated_user.into_active_model();
    active_model.id = Set(id);

    let updated_user = active_model
        .update(db)
        .await
        .map_err(internal_error_with_log!())?;

    Ok(Json(UserResponse::from(updated_user)))
}

pub fn get_routes(state: &AppStateRef) -> Router {
    Router::new()
        // ---
        .route("/", post(create_user))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            create_user_middleware,
        ))
        // ---
        .route("/", get(get_users))
        .route("/:id", get(get_user))
        .route("/:id", delete(delete_user))
        .route("/:id", patch(update_user))
        .route_layer(middleware::from_fn(admin_middleware))
        // ---
        .with_state(state.clone())
}
