use std::path::PathBuf;

use tokio::{
    fs::{self, File},
    io::{self, AsyncWriteExt},
    sync::broadcast::error::RecvError,
};
use tracing::{Instrument, instrument};

use crate::services::ProcessRef;

pub struct LogService {
    log_path: PathBuf,
}

impl LogService {
    pub fn new(log_path: PathBuf) -> Self {
        Self { log_path }
    }

    pub fn get_log_path(&self, id: u64) -> PathBuf {
        self.log_path.join(format!("{}.log", id))
    }

    pub async fn get_log_begin(&self, id: u64) -> Result<u64, io::Error> {
        let log_file = File::open(self.get_log_path(id)).await?;
        Ok(log_file.metadata().await?.len())
    }

    #[instrument(skip(process_ref, self), parent = None)]
    pub async fn begin_log(&self, id: u64, process_ref: ProcessRef) -> Result<(), io::Error> {
        // prepare for streams
        let (mut stdout, mut stderr) = {
            let process = process_ref.read().await;
            (process.get_stdout(), process.get_stderr())
        };

        // prepare for log file
        let log_path = self.get_log_path(id);
        if log_path.exists() {
            fs::remove_file(&log_path).await?;
        }
        let file = File::create_new(log_path).await?;

        tokio::spawn(
            async move {
                let mut file = file;

                tracing::info!("Logger for process {} started", id);

                loop {
                    let data = tokio::select! {
                        data = stdout.as_mut().unwrap().recv(), if stdout.is_some() => data,
                        data = stderr.as_mut().unwrap().recv(), if stderr.is_some() => data,
                        else => break,
                    };

                    match data {
                        Err(e) => match e {
                            RecvError::Lagged(_) => {
                                continue;
                            }
                            RecvError::Closed => break, // process exited
                        },

                        Ok(message) => {
                            if let Err(e) = file.write(&message).await {
                                tracing::error!("Logger for process {} error: {}", id, e);
                                break;
                            }
                        }
                    }
                }

                tracing::info!("Logger for process {} exited", id);
            }
            .instrument(tracing::info_span!(parent: None, "log worker", id)),
        );

        Ok(())
    }
}
