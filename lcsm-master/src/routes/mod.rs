use axum::Router;

mod files;
mod instances;
mod processes;
mod slave_proxy;
mod slaves;
mod users;
use crate::AppStateRef;

pub fn get_routes(state: &AppStateRef) -> Router {
    Router::new()
        .nest("/user", users::get_routes(state))
        .nest("/slave", slaves::get_routes(state))
        .nest("/process", processes::get_routes(state))
        .nest("/file", files::get_routes(state))
        .nest("/instance", instances::get_routes(state))
}
