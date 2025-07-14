use axum::http::StatusCode;
use log::error;
use std::fmt::Display;

pub fn some_status_code_with_log<E: Display>(
    status_code: StatusCode,
) -> impl FnOnce(E) -> StatusCode {
    move |e| {
        error!("{}", e);
        status_code
    }
}

pub fn internal_error_with_log<E: Display>() -> impl FnOnce(E) -> StatusCode {
    some_status_code_with_log(StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn bad_request_with_log<E: Display>() -> impl FnOnce(E) -> StatusCode {
    some_status_code_with_log(StatusCode::BAD_REQUEST)
}
