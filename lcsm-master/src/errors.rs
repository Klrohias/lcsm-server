#[macro_export]
macro_rules! status_code_with_log {
    ($status_code:expr) => {
        |e| {
            use log::error;

            error!("{}", e);
            $status_code
        }
    };
}

#[macro_export]
macro_rules! internal_error_with_log {
    () => {{
        use axum::http::StatusCode;
        $crate::status_code_with_log!(StatusCode::INTERNAL_SERVER_ERROR)
    }};
}

#[macro_export]
macro_rules! bad_request_with_log {
    () => {{
        use axum::http::StatusCode;
        $crate::status_code_with_log!(StatusCode::BAD_REQUEST)
    }};
}
#[macro_export]
macro_rules! trace_error {
    ($msg:expr, $status_code:expr) => {
        |e| {
            use tracing::error;

            error!("{}: {}", $msg, e);
            $status_code
        }
    };
}

pub use bad_request_with_log;
pub use internal_error_with_log;
pub use status_code_with_log;
pub use trace_error;
