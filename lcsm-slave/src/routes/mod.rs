use axum::Router;

use crate::AppStateRef;

mod instances;
mod processes;

pub fn get_routes(state_ref: &AppStateRef) -> Router {
    Router::new()
        .nest("/instances", instances::get_routes(state_ref))
        .nest("/processes", processes::get_routes(state_ref))
}
