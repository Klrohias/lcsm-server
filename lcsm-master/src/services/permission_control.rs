use std::sync::Arc;

use axum::{
    Extension,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
};
use tracing::instrument;

use crate::{
    services::{UserServiceRef, auth::Claims},
    trace_error,
};

pub type PermissionServiceRef = Arc<PermissionService>;

pub struct PermissionService {
    user_service: UserServiceRef,
}

impl PermissionService {
    pub fn new(user_service: UserServiceRef) -> Self {
        Self { user_service }
    }

    pub async fn has_permission_to_instance(
        &self,
        user_id: i32,
        _slave_id: i32,
        _instance_id: i32,
    ) -> bool {
        let user = match self
            .user_service
            .find_user_by_id(user_id)
            .await
            .map_err(trace_error!("check permission"))
        {
            Err(_) => return false,
            Ok(v) => v,
        };

        // if user is admin, allow
        if user.user_type == "administrator" {
            return true;
        }

        // if user is banned, deny
        if user.banned {
            return false;
        }

        // check the policy
        todo!()
    }

    pub async fn is_administrator(&self, user_id: i32) -> bool {
        let result = self
            .user_service
            .find_user_by_id(user_id)
            .await
            .map_err(trace_error!("check admin"));

        match result {
            Ok(v) => v.user_type == "administrator",
            Err(_) => false,
        }
    }
}

#[instrument(skip_all, fields(claims))]
pub async fn admin_middleware(
    Extension(claims): Extension<Claims>,
    State(permission_service): State<PermissionServiceRef>,
    request: Request,
    next: Next,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let is_admin = permission_service.is_administrator(claims.id).await;

    if !is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
