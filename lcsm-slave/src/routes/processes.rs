use std::{ffi::OsString, path::PathBuf};

use axum::{
    Json, Router,
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get, put},
};

use sea_orm::EntityTrait;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, SeekFrom},
    sync::{broadcast::error::RecvError, mpsc, oneshot},
    task::JoinError,
};
use tracing::{Instrument, instrument};

use crate::{
    AppStateRef,
    entities::instance,
    errors::trace_error,
    services::{ProcessRef, ProcessRuntimeStatus, ProcessState},
};

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

pub fn get_routes(state_ref: &AppStateRef) -> Router {
    Router::new()
        .route(
            "/{id}",
            put(start_process).delete(kill_process).get(process_state),
        )
        .route("/{id}/terminal", any(terminal_ws_connect))
        .route("/{id}/logs", get(fetch_process_log))
        .with_state(state_ref.clone())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessStateResponse {
    status: &'static str,
    is_running: bool,
    pid: Option<u32>,
    started_at: Option<u64>,
    uptime_ms: Option<u64>,
    cpu_percent: Option<f32>,
    memory_bytes: Option<u64>,
}

impl From<ProcessRuntimeStatus> for ProcessStateResponse {
    fn from(value: ProcessRuntimeStatus) -> Self {
        Self {
            status: value.status.as_str(),
            is_running: value.is_running(),
            pid: value.pid,
            started_at: value.started_at,
            uptime_ms: value.uptime_ms,
            cpu_percent: value.cpu_percent,
            memory_bytes: value.memory_bytes,
        }
    }
}

async fn get_alive_process(state: &AppStateRef, id: u64) -> Option<ProcessRef> {
    let process = state.process_manager.get_process(id).await?;

    let state = {
        let process_read = process.read().await;
        process_read.state().await
    };
    if state == ProcessState::Dead {
        return None;
    }

    Some(process)
}

#[instrument(skip(state))]
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
    let db = &state.database;
    let the_instance = instance::Entity::find_by_id(
        i32::try_from(id).map_err(trace_error!("parse id", StatusCode::BAD_REQUEST))?,
    )
    .one(db)
    .await
    .map_err(trace_error!(
        "one from db",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?
    .ok_or(StatusCode::NOT_FOUND)?;

    let arguments = the_instance
        .arguments
        .lines()
        .map(|x| x.into())
        .collect::<Vec<OsString>>();

    let work_dir = {
        let wd = the_instance.work_dir.trim();
        if wd.is_empty() {
            // default: the instance's own data directory
            state.instances_path.join(the_instance.id.to_string())
        } else {
            let path = PathBuf::from(wd);
            if path.is_absolute() {
                path
            } else {
                state
                    .instances_path
                    .join(the_instance.id.to_string())
                    .join(path)
            }
        }
    };

    let process_ref = state
        .process_manager
        .new_process(
            id,
            the_instance.launch_command,
            arguments,
            work_dir,
            the_instance.use_shell,
        )
        .await
        .map_err(trace_error!(
            "spawn process",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?;

    _ = state.log_manager.begin_log(id, process_ref).await;

    Ok(())
}

#[instrument(skip(state))]
async fn kill_process(
    State(state): State<AppStateRef>,
    Path(id): Path<u64>,
) -> Result<(), StatusCode> {
    let process = get_alive_process(&state, id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    process.read().await.kill().await.map_err(trace_error!(
        "kill process",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    Ok(())
}

#[instrument(skip(state))]
async fn process_state(
    State(state): State<AppStateRef>,
    Path(id): Path<u64>,
) -> Result<Json<ProcessStateResponse>, StatusCode> {
    instance::Entity::find_by_id(i32::try_from(id).map_err(trace_error!(
        "parse id",
        StatusCode::BAD_REQUEST
    ))?)
    .one(&state.database)
    .await
    .map_err(trace_error!(
        "one from db",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?
    .ok_or(StatusCode::NOT_FOUND)?;

    let runtime = state.process_manager.get_runtime_status(id).await;
    Ok(Json(runtime.into()))
}

#[instrument(skip(state))]
async fn terminal_ws_connect(
    State(state): State<AppStateRef>,
    Path(id): Path<u64>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, StatusCode> {
    let process = get_alive_process(&state, id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    let log_begin = state
        .log_manager
        .get_log_begin(id)
        .await
        .map_err(trace_error!("get_log_begin", 0));

    let log_begin = log_begin.unwrap_or_else(|_| log_begin.unwrap_err());

    Ok((
        [("X-Log-Begin", log_begin)],
        ws.on_upgrade(move |ws| terminal_ws_handler(ws, id, process)),
    ))
}

#[instrument(skip(socket, process))]
async fn terminal_ws_handler(socket: WebSocket, id: u64, process: ProcessRef) {
    let (mut socket_write, mut socket_read) = socket.split();
    let (exit_sender, mut exit_receiver) = oneshot::channel::<()>();
    let (output_sender, mut output_receiver) = mpsc::channel(16);

    let mut stdout = { process.read().await.get_stdout() };
    let mut stderr = { process.read().await.get_stderr() };

    tracing::info!("Socket for process {} opened", id);

    let mut send_task = tokio::spawn(
        async move {
            use anyhow::Ok;

            loop {
                tokio::select! {
                    data = output_receiver.recv(), if !output_receiver.is_closed() => {
                        if data.is_none() {
                            continue; // now closed :)
                        }

                        socket_write.send(Message::binary(data.unwrap())).await?;
                    }
                    _ = &mut exit_receiver => {
                        socket_write.close().await?;
                        break Ok(());
                    }
                }
            }
        }
        .instrument(tracing::info_span!("send task")),
    );

    let mut output_task = tokio::spawn(
        async move {
            use anyhow::Ok;

            loop {
                let data = tokio::select! {
                    data = stdout.as_mut().unwrap().recv(), if stdout.is_some() => data,
                    data = stderr.as_mut().unwrap().recv(), if stderr.is_some() => data,
                    else => {
                        break Ok(());
                    }
                };

                if let Err(e) = data {
                    if let RecvError::Lagged(_) = e {
                        continue;
                    } else {
                        break Ok(());
                    }
                }

                output_sender.send(data.unwrap()).await?;
            }
        }
        .instrument(tracing::info_span!("output task")),
    );

    let mut input_task = tokio::spawn(
        async move {
            use anyhow::Ok;

            let process_write = process.read().await;
            let stdin = process_write.get_stdin();
            drop(process_write);

            if stdin.is_none() {
                return Ok(());
            }
            let stdin = stdin.unwrap();

            while let Some(message) = socket_read.next().await {
                let message = message?;
                let Some(data) = terminal_input_bytes(message) else {
                    continue;
                };

                stdin.send(data).await?;
            }

            Ok(())
        }
        .instrument(tracing::info_span!("input task")),
    );

    let error = tokio::select! {
        r = &mut send_task => just_get_error(r),
        r = &mut output_task => just_get_error(r),
        r = &mut input_task => just_get_error(r),
    };

    if let Some(err) = error {
        tracing::error!("Socket for process {} closed with error: {}", id, err);
    }

    // close connection
    _ = exit_sender.send(());
    _ = send_task.await;
    tracing::info!("Socket for process {} closed", id);
}

fn terminal_input_bytes(message: Message) -> Option<Vec<u8>> {
    match message {
        Message::Text(text) => Some(text.as_bytes().to_vec()),
        Message::Binary(data) => Some(data.to_vec()),
        Message::Close(_) => None,
        Message::Ping(_) | Message::Pong(_) => None,
    }
}

fn just_get_error(r: Result<Result<(), anyhow::Error>, JoinError>) -> Option<anyhow::Error> {
    match r {
        Ok(result) => match result {
            Ok(_) => None,
            Err(e) => Some(e.into()),
        },
        Err(e) => Some(e.into()),
    }
}

#[derive(Debug, Deserialize)]
struct LogQuery {
    before: Option<u64>,
    limit: Option<usize>,
    tail: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogChunkResponse {
    content: String,
    has_more: bool,
    begin_offset: u64,
}

#[instrument(skip(state))]
async fn fetch_process_log(
    State(state): State<AppStateRef>,
    Path(id): Path<u64>,
    Query(query): Query<LogQuery>,
) -> Result<Json<LogChunkResponse>, StatusCode> {
    let log_path = state.log_path.join(format!("{id}.log"));
    let mut file = File::open(&log_path).await.map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
        _ => trace_error!("open process log", StatusCode::INTERNAL_SERVER_ERROR)(error),
    })?;

    let file_size = file
        .metadata()
        .await
        .map_err(trace_error!("read process log metadata", StatusCode::INTERNAL_SERVER_ERROR))?
        .len();

    let limit = query.limit.unwrap_or(8 * 1024).clamp(1, 256 * 1024);
    let before = query.before.unwrap_or(file_size).min(file_size);
    let start = if let Some(tail) = query.tail {
        file_size.saturating_sub(u64::try_from(tail).unwrap_or(u64::MAX))
    } else {
        before.saturating_sub(u64::try_from(limit).unwrap_or(u64::MAX))
    };

    file.seek(SeekFrom::Start(start))
        .await
        .map_err(trace_error!("seek process log", StatusCode::INTERNAL_SERVER_ERROR))?;

    let to_read = usize::try_from(before.saturating_sub(start))
        .unwrap_or(limit)
        .min(limit);
    let mut buf = vec![0; to_read];
    let read = file
        .read(&mut buf)
        .await
        .map_err(trace_error!("read process log", StatusCode::INTERNAL_SERVER_ERROR))?;
    buf.truncate(read);

    let content = String::from_utf8_lossy(&buf).into_owned();
    let begin_offset = start;
    let has_more = begin_offset > 0;

    Ok(Json(LogChunkResponse {
        content,
        has_more,
        begin_offset,
    }))
}
