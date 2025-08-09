#[macro_export]
macro_rules! trace_error {
    ($msg:expr) => {
        |e| {
            use tracing::error;

            error!("{}: {}", $msg, e);
            ()
        }
    };

    ($msg:expr, $($resp:expr),+) => {
        |e| {
            use tracing::error;

            error!("{}: {}", $msg, e);
            $crate::api_error!($($resp),+)
        }
    };
}

#[macro_export]
macro_rules! api_error {
    ($status_code:expr) => {{
        use axum::response::{IntoResponse, Json};
        (
            $status_code,
            Json($crate::errors::ErrorResponse {
                message: None,
                status_code: $status_code.as_u16(),
            }),
        )
            .into_response()
    }};

    ($msg:expr, $status_code:expr) => {{
        use axum::response::{IntoResponse, Json};
        (
            $status_code,
            Json($crate::errors::ErrorResponse {
                message: Some($msg),
                status_code: $status_code.as_u16(),
            }),
        )
            .into_response()
    }};
}

pub use api_error;
use serde::Serialize;
pub use trace_error;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub status_code: u16,
}
