use std::sync::Arc;

use anyhow::Result;
use axum::{
    Extension,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::internal_error_with_log;

pub type AuthServiceRef = Arc<Mutex<AuthService>>;

pub struct AuthService {
    jwt_secret: String,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> AuthServiceRef {
        Arc::new(Mutex::new(AuthService { jwt_secret }))
    }

    pub fn get_jwt_secret(&self) -> &String {
        &self.jwt_secret
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub user_type: String,
    pub exp: usize,
}

pub async fn jwt_middleware(
    State(state): State<AuthServiceRef>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(internal_error_with_log!())?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];

    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.lock().await.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(internal_error_with_log!())?;

    request.extensions_mut().insert(token_data.claims);
    Ok(next.run(request).await)
}
