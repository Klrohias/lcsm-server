use axum::{http::StatusCode, response::Response};
use reqwest::Method;
use sea_orm::EntityTrait;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

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
