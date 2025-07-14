use axum::Router;

use crate::AppStateRef;

mod instance;

pub fn get_routes(state: &AppStateRef) -> Router {
    Router::new().nest("/instances", instance::get_routes(state))
}
