use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{api_error, trace_error};

pub type AuthServiceRef = Arc<AuthService>;

pub struct AuthService {
    decoding_key: DecodingKey,
    encoding_key: EncodingKey,
    validation: Validation,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> AuthServiceRef {
        let jwt_secret = jwt_secret.as_bytes();
        Arc::new(AuthService {
            decoding_key: DecodingKey::from_secret(jwt_secret),
            encoding_key: EncodingKey::from_secret(jwt_secret),
            validation: Validation::new(Algorithm::HS256),
        })
    }

    pub fn decode_claims(&self, token: impl AsRef<str>) -> Result<Claims> {
        Ok(decode(token.as_ref(), &self.decoding_key, &self.validation)?.claims)
    }

    fn encode_claims(&self, claims: Claims) -> Result<String> {
        Ok(encode(&Header::default(), &claims, &self.encoding_key)?)
    }

    pub fn create_jwt(&self, user_id: i32, user_email: &str) -> Result<String> {
        let expiration = SystemTime::now()
            .checked_add(Duration::from_secs(60 * 60 * 24 * 7))
            .ok_or_else(|| anyhow::anyhow!("Failed to add time"))?
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let claims = Claims {
            sub: user_email.to_owned(),
            id: user_id,
            exp: expiration as usize,
        };

        self.encode_claims(claims)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub id: i32,
    pub exp: usize,
}

pub async fn jwt_middleware(
    State(state): State<AuthServiceRef>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let auth_header = headers
        .get("authorization")
        .ok_or(api_error!(StatusCode::UNAUTHORIZED))?
        .to_str()
        .map_err(trace_error!(
            "decode authorization header",
            StatusCode::UNAUTHORIZED
        ))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(api_error!(StatusCode::UNAUTHORIZED));
    }

    let token = &auth_header[7..];

    let claims = state
        .decode_claims(token)
        .map_err(trace_error!("decode jwt claims", StatusCode::UNAUTHORIZED))?;

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}
