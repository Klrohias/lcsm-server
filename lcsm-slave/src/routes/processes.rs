use std::ffi::OsString;

use axum::{
    Router,
    extract::{
        Path, Request, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get, put},
};
use log::{error, info, warn};
use sea_orm::EntityTrait;
use tokio::{
    sync::{broadcast::error::RecvError, mpsc, oneshot},
    task::JoinError,
};
use tower::ServiceExt;
use tower_http::services::ServeFile;

use crate::{
    AppStateRef,
    entities::instance,
    errors::{bad_request_with_log, internal_error_with_log},
    services::{ProcessRef, ProcessState},
    something_with_log,
};

use futures::{SinkExt, StreamExt};

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

async fn get_alive_process(state: &AppStateRef, id: u64) -> Option<ProcessRef> {
    let process = state.process_manager.get_process(id).await;
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
    let the_instance =
        instance::Entity::find_by_id(i32::try_from(id).map_err(bad_request_with_log!())?)
            .one(db)
            .await
            .map_err(internal_error_with_log!())?
            .ok_or(StatusCode::NOT_FOUND)?;

    let arguments = the_instance
        .arguments
        .lines()
        .map(|x| x.into())
        .collect::<Vec<OsString>>();

    let process_ref = state
        .process_manager
        .new_process(
            id,
            the_instance.launch_command,
            arguments,
            the_instance.work_dir,
            the_instance.use_shell,
        )
        .await
        .map_err(internal_error_with_log!())?;

    _ = state.log_manager.begin_log(id, process_ref).await;

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
        .map_err(internal_error_with_log!())?;

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
        .map_err(something_with_log!(0 as u64));

    let log_begin = log_begin.unwrap_or_else(|_| log_begin.unwrap_err());

    Ok((
        [("X-Log-Begin", log_begin)],
        ws.on_upgrade(move |ws| terminal_ws_handler(ws, id, process)),
    ))
}

async fn terminal_ws_handler(socket: WebSocket, id: u64, process: ProcessRef) {
    let (mut socket_write, mut socket_read) = socket.split();
    let (exit_sender, mut exit_receiver) = oneshot::channel::<()>();
    let (output_sender, mut output_receiver) = mpsc::channel(16);

    let mut stdout = { process.read().await.get_stdout() };
    let mut stderr = { process.read().await.get_stderr() };

    info!("Socket for process {} opened", id);

    let send_task = tokio::spawn(async move {
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
    });

    let output_task = tokio::spawn(async move {
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
                if let RecvError::Lagged(l) = e {
                    warn!("Receiver for process {} lagged {}", id, l);
                    continue;
                } else {
                    break Err(e.into());
                }
            }

            output_sender.send(data.unwrap()).await?;
        }
    });

    let input_task = { process.read().await.get_stdin() }.map(|sender| {
        tokio::spawn(async move {
            use anyhow::Ok;
            // redirect stdin
            while let Some(message) = socket_read.next().await {
                if let Err(e) = message {
                    return Err(e.into());
                }

                let message = message.unwrap();

                // send
                if let Err(e) = sender.send(message.into_data().to_vec()).await {
                    return Err(e.into());
                }
            }

            Ok(())
        })
    });

    let error = tokio::select! {
        r = output_task => just_get_error(r),
        r = input_task.unwrap(), if input_task.is_some() => just_get_error(r)
    };

    if let Some(err) = error {
        error!("Stdio for process {} closed with error: {}", id, err);
    }

    // close connection
    _ = exit_sender.send(());
    _ = send_task.await;
    info!("Socket for process {} closed", id);
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

async fn fetch_process_log(
    Path(id): Path<u64>,
    State(state): State<AppStateRef>,
    request: Request,
) -> Result<impl IntoResponse, StatusCode> {
    let log_path = state.log_manager.get_log_path(id);
    if !log_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(ServeFile::new(log_path).oneshot(request).await)
}
