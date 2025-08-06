use axum::{
    Extension, Router,
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Json,
    routing::{delete, get, patch, post},
};
use bcrypt::{DEFAULT_COST, hash, verify};
use json_patch::Patch;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Unchanged, ColumnTrait, EntityTrait, IntoActiveModel,
    ModelTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tracing::instrument;

use crate::{
    AppStateRef,
    entities::user,
    services::auth,
    trace_error,
    transfer::{PaginationOptions, PaginationResponse},
};

pub fn get_routes(state: &AppStateRef) -> Router {
    Router::new()
        // ---
        .route("/", post(create_user))
        .route_layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn_with_state(
                    state.auth_service.clone(),
                    auth::jwt_middleware,
                ))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    create_user_middleware,
                )),
        )
        // ---
        .route("/", get(get_users))
        .route("/:id", get(get_user))
        .route("/:id", delete(delete_user))
        .route("/:id", patch(update_user))
        .route_layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn_with_state(
                    state.auth_service.clone(),
                    auth::jwt_middleware,
                ))
                .layer(middleware::from_fn(auth::admin_middleware)),
        )
        // ---
        .route("/me", get(get_current_user))
        .route_layer(middleware::from_fn_with_state(
            state.auth_service.clone(),
            auth::jwt_middleware,
        ))
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
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let db = &state.database_connection;

    // Check if any users exist
    let user_count = user::Entity::find().count(db).await.map_err(trace_error!(
        "check user exists",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    // Allow first user creation or admin users
    if user_count == 0 || claims.user_type == "administrator" {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
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
) -> Result<Json<UserResponse>, StatusCode> {
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
) -> Result<Json<UserResponse>, StatusCode> {
    let db = &state.database_connection;

    let user = user::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(trace_error!("find user", StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse::from(user)))
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
) -> Result<Json<PaginationResponse<UserResponse>>, StatusCode> {
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
) -> Result<StatusCode, StatusCode> {
    let db = &state.database_connection;

    let user = user::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(trace_error!("find user", StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or(StatusCode::NOT_FOUND)?;

    user.delete(db).await.map_err(trace_error!(
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
) -> Result<Json<UserResponse>, StatusCode> {
    let db = &state.database_connection;

    let user = user::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(trace_error!("find user", StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or(StatusCode::NOT_FOUND)?;

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
) -> Result<Json<UserResponse>, StatusCode> {
    let db = &state.database_connection;

    let user = user::Entity::find_by_id(
        i32::try_from(claims.id)
            .map_err(trace_error!("parse id", StatusCode::INTERNAL_SERVER_ERROR))?,
    )
    .one(db)
    .await
    .map_err(trace_error!("find user", StatusCode::INTERNAL_SERVER_ERROR))?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse::from(user)))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub access_token: String,
}

#[instrument(skip_all, fields(user.email = request.email))]
pub async fn login(
    State(state): State<AppStateRef>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let db = &state.database_connection;
    let auth_service = state.auth_service.read().await;

    let user = user::Entity::find()
        .filter(user::Column::Email.eq(request.email))
        .one(db)
        .await
        .map_err(trace_error!("find user", StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify(&request.password, &user.password_hash).map_err(trace_error!(
        "bcrypt verify",
        StatusCode::INTERNAL_SERVER_ERROR
    ))? {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = auth_service
        .create_jwt(user.id, &user.email, &user.user_type)
        .map_err(trace_error!(
            "create_jwt",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?;

    Ok(Json(LoginResponse {
        access_token: token,
    }))
}
