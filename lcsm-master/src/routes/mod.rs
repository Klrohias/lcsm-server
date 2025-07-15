use axum::Router;

mod users;
use crate::AppStateRef;

pub fn get_routes(state: &AppStateRef) -> Router {
    Router::new().nest("/user/", users::get_routes(state))
}
