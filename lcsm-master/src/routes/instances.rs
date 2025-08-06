use axum::{Router, middleware};

use crate::{AppStateRef, services::auth};

pub fn get_routes(state: &AppStateRef) -> Router {
    Router::new()
        .route_layer(middleware::from_fn_with_state(
            state.auth_service.clone(),
            auth::jwt_middleware,
        ))
        .with_state(state.clone())
}
