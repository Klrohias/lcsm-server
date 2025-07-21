use axum::{Router, extract::State, routing::get};

use crate::AppStateRef;

pub fn get_routes(state_ref: &AppStateRef) -> Router {
    Router::new()
        .route("/images", get(get_images))
        .with_state(state_ref.clone())
}

async fn get_images(State(_state): State<AppStateRef>) {}
