use axum::Router;

use crate::AppStateRef;

mod instances;
mod processes;

pub fn get_routes(state_ref: &AppStateRef) -> Router {
    Router::new()
        .nest("/instance", instances::get_routes(state_ref))
        .nest("/process", processes::get_routes(state_ref))
}
