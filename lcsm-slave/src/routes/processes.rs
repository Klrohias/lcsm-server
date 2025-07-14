use std::ffi::OsString;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::put,
};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};

use crate::{
    AppStateRef,
    entities::instance,
    errors::{bad_request_with_log, internal_error_with_log},
    services::{ProcessRef, ProcessState},
    transfer::PaginationOptions,
};
use axum_extra::extract::Query as ExtraQuery;

pub fn get_routes(state_ref: &AppStateRef) -> Router {
    Router::new()
        .route(
            "/{id}",
            put(start_process).delete(kill_process).get(process_state),
        )
        .with_state(state_ref.clone())
}

#[derive(Deserialize)]
struct ProcessesQuery {
    pub ids: Option<Vec<u64>>,
}

async fn get_alive_process(state: &AppStateRef, id: u64) -> Option<ProcessRef> {
    let process = state.pm.get_process(id).await;
    if process.is_none() {
        return None;
    }

    let process = process.unwrap();
    let state = {
        let process_read = process.read().await;
        process_read.state().await
    };
    if state == ProcessState::Dead {
        return None;
    }

    return Some(process);
}

async fn get_processes(
    State(state): State<AppStateRef>,
    Query(pagination): Query<PaginationOptions>,
    ExtraQuery(query): ExtraQuery<ProcessesQuery>,
) {
}

async fn start_process(
    State(state): State<AppStateRef>,
    Path(id): Path<u64>,
) -> Result<(), StatusCode> {
    // is the process existed?
    let process = get_alive_process(&state, id).await;

    if process.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    // start process
    let db = &state.db;
    let the_instance =
        instance::Entity::find_by_id(i32::try_from(id).map_err(bad_request_with_log())?)
            .one(db)
            .await
            .map_err(internal_error_with_log())?
            .ok_or(StatusCode::NOT_FOUND)?;

    state
        .pm
        .new_process(
            id,
            the_instance.launch_command,
            &[] as &[OsString],
            the_instance.work_dir,
        )
        .await
        .map_err(internal_error_with_log())?;

    Ok(())
}

async fn kill_process(
    State(state): State<AppStateRef>,
    Path(id): Path<u64>,
) -> Result<(), StatusCode> {
    let process = get_alive_process(&state, id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    process
        .read()
        .await
        .kill()
        .await
        .map_err(internal_error_with_log())?;

    Ok(())
}

async fn process_state(
    State(state): State<AppStateRef>,
    Path(id): Path<u64>,
) -> Result<(), StatusCode> {
    get_alive_process(&state, id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(())
}
