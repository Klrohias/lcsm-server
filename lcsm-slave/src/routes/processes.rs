use axum::{
    Router,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::{AppStateRef, transfer::PaginationOptions};
use axum_extra::extract::Query as ExtraQuery;

pub fn get_routes(state_ref: &AppStateRef) -> Router {
    Router::new().with_state(state_ref.clone())
}

#[derive(Deserialize)]
struct ProcessesQuery {
    ids: Option<Vec<u64>>,
}

async fn get_processes(
    State(state): State<AppStateRef>,
    Query(pagination): Query<PaginationOptions>,
    ExtraQuery(query): ExtraQuery<ProcessesQuery>,
) {
}
