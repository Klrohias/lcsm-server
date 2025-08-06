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

pub use trace_error;
