use axum::{
    extract::ws::{CloseFrame, Message, WebSocket},
    http::StatusCode,
    response::Response,
};
use futures::{SinkExt, StreamExt};
use reqwest::Method;
use sea_orm::EntityTrait;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use tokio::task::JoinError;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        self,
        client::IntoClientRequest,
        protocol::frame::coding::CloseCode,
    },
};

use crate::{AppStateRef, api_error, entities::slave, trace_error};

pub async fn find_slave(state: &AppStateRef, id: i32) -> Result<slave::Model, Response> {
    slave::Entity::find_by_id(id)
        .one(&state.database_connection)
        .await
        .map_err(trace_error!(
            "find slave",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?
        .ok_or(api_error!(StatusCode::NOT_FOUND))
}

pub fn slave_url(slave: &slave::Model, path: &str) -> String {
    format!(
        "{}/{}",
        slave.slave_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

pub fn slave_ws_url(slave: &slave::Model, path: &str) -> String {
    let base = slave.slave_url.trim_end_matches('/');
    let base = if let Some(stripped) = base.strip_prefix("http://") {
        format!("ws://{stripped}")
    } else if let Some(stripped) = base.strip_prefix("https://") {
        format!("wss://{stripped}")
    } else {
        base.to_owned()
    };

    format!("{}/{}", base, path.trim_start_matches('/'))
}

fn response_status_error(status: reqwest::StatusCode) -> Response {
    api_error!(StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY))
}

pub async fn proxy_json<T, Q>(
    slave: &slave::Model,
    method: Method,
    path: &str,
    query: Option<&Q>,
    body: Option<Value>,
) -> Result<T, Response>
where
    T: DeserializeOwned,
    Q: Serialize + ?Sized,
{
    let client = reqwest::Client::new();
    let mut request = client
        .request(method, slave_url(slave, path))
        .bearer_auth(&slave.slave_token);

    if let Some(query) = query {
        request = request.query(query);
    }
    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = request
        .send()
        .await
        .map_err(trace_error!("request slave", StatusCode::BAD_GATEWAY))?;
    let status = response.status();
    if !status.is_success() {
        return Err(response_status_error(status));
    }

    response.json::<T>().await.map_err(trace_error!(
        "decode slave response",
        StatusCode::BAD_GATEWAY
    ))
}

pub async fn proxy_text<Q>(
    slave: &slave::Model,
    method: Method,
    path: &str,
    query: Option<&Q>,
) -> Result<String, Response>
where
    Q: Serialize + ?Sized,
{
    let client = reqwest::Client::new();
    let mut request = client
        .request(method, slave_url(slave, path))
        .bearer_auth(&slave.slave_token);

    if let Some(query) = query {
        request = request.query(query);
    }

    let response = request
        .send()
        .await
        .map_err(trace_error!("request slave", StatusCode::BAD_GATEWAY))?;
    let status = response.status();
    if !status.is_success() {
        return Err(response_status_error(status));
    }

    response.text().await.map_err(trace_error!(
        "decode slave response",
        StatusCode::BAD_GATEWAY
    ))
}

pub async fn proxy_empty(
    slave: &slave::Model,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<StatusCode, Response> {
    let client = reqwest::Client::new();
    let mut request = client
        .request(method, slave_url(slave, path))
        .bearer_auth(&slave.slave_token);
    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = request
        .send()
        .await
        .map_err(trace_error!("request slave", StatusCode::BAD_GATEWAY))?;
    let status = response.status();
    if !status.is_success() {
        return Err(response_status_error(status));
    }

    Ok(StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::OK))
}

pub async fn proxy_websocket(slave: slave::Model, path: String, socket: WebSocket) {
    if let Err(error) = proxy_websocket_inner(&slave, &path, socket).await {
        tracing::warn!(slave_id = slave.id, %path, %error, "proxy websocket closed with error");
    }
}

async fn proxy_websocket_inner(
    slave: &slave::Model,
    path: &str,
    socket: WebSocket,
) -> anyhow::Result<()> {
    let mut request = slave_ws_url(slave, path).into_client_request()?;
    request.headers_mut().insert(
        "Authorization",
        format!("Bearer {}", slave.slave_token).parse()?,
    );

    let (upstream, _) = connect_async(request).await?;
    let (mut browser_write, mut browser_read) = socket.split();
    let (mut upstream_write, mut upstream_read) = upstream.split();

    let mut browser_to_upstream = tokio::spawn(async move {
        while let Some(message) = browser_read.next().await {
            let message = message?;
            let Some(message) = axum_to_tungstenite(message) else {
                continue;
            };
            upstream_write.send(message).await?;
        }

        anyhow::Ok(())
    });

    let mut upstream_to_browser = tokio::spawn(async move {
        while let Some(message) = upstream_read.next().await {
            let message = message?;
            let Some(message) = tungstenite_to_axum(message) else {
                continue;
            };
            browser_write.send(message).await?;
        }

        anyhow::Ok(())
    });

    let error = tokio::select! {
        result = &mut browser_to_upstream => join_result_error(result),
        result = &mut upstream_to_browser => join_result_error(result),
    };

    browser_to_upstream.abort();
    upstream_to_browser.abort();

    if let Some(error) = error {
        return Err(error);
    }

    Ok(())
}

fn join_result_error(result: Result<anyhow::Result<()>, JoinError>) -> Option<anyhow::Error> {
    match result {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(error),
        Err(error) => Some(error.into()),
    }
}

fn axum_to_tungstenite(message: Message) -> Option<tungstenite::Message> {
    match message {
        Message::Text(text) => Some(tungstenite::Message::Text(text.to_string().into())),
        Message::Binary(data) => Some(tungstenite::Message::Binary(data)),
        Message::Ping(data) => Some(tungstenite::Message::Ping(data)),
        Message::Pong(data) => Some(tungstenite::Message::Pong(data)),
        Message::Close(frame) => Some(tungstenite::Message::Close(
            frame.map(|frame| tungstenite::protocol::CloseFrame {
                code: CloseCode::from(frame.code),
                reason: frame.reason.to_string().into(),
            }),
        )),
    }
}

fn tungstenite_to_axum(message: tungstenite::Message) -> Option<Message> {
    match message {
        tungstenite::Message::Text(text) => Some(Message::Text(text.to_string().into())),
        tungstenite::Message::Binary(data) => Some(Message::Binary(data)),
        tungstenite::Message::Ping(data) => Some(Message::Ping(data)),
        tungstenite::Message::Pong(data) => Some(Message::Pong(data)),
        tungstenite::Message::Close(frame) => Some(Message::Close(frame.map(|frame| CloseFrame {
            code: frame.code.into(),
            reason: frame.reason.to_string().into(),
        }))),
        tungstenite::Message::Frame(_) => None,
    }
}
