use std::{collections::HashMap, ffi::OsStr, path::Path, process::Stdio, sync::Arc};

use anyhow::Result;
use log::warn;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    process::{Child, Command},
    sync::{RwLock, broadcast, mpsc},
};

use crate::transfer::{BinarySequence, redirect_input, redirect_output};

fn create_output_redirect(
    output: impl AsyncRead + Unpin + Sync + Send + 'static,
) -> broadcast::Receiver<BinarySequence> {
    let (tx, rx) = broadcast::channel(8);

    tokio::spawn(async move {
        if let Err(e) = redirect_output(output, tx).await {
            warn!("{}", e);
        }
    });

    return rx;
}

fn create_input_redirect(
    input: impl AsyncWrite + Unpin + Sync + Send + 'static,
) -> mpsc::Sender<BinarySequence> {
    let (tx, rx) = mpsc::channel(8);

    tokio::spawn(async move {
        if let Err(e) = redirect_input(input, rx).await {
            warn!("{}", e);
        }
    });

    return tx;
}

pub struct Process {
    child: Arc<RwLock<Child>>,

    stdout: Option<broadcast::Receiver<Vec<u8>>>,
    stderr: Option<broadcast::Receiver<Vec<u8>>>,
    stdin: Option<mpsc::Sender<Vec<u8>>>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ProcessState {
    Alive,
    Dead,
}

impl Process {
    pub fn new(mut child: Child) -> Self {
        let stdout = child.stdout.take().map(create_output_redirect);
        let stderr = child.stderr.take().map(create_output_redirect);
        let stdin = child.stdin.take().map(create_input_redirect);

        let child = Arc::new(RwLock::new(child));

        Self {
            child,
            stdout,
            stderr,
            stdin,
        }
    }

    pub async fn kill(&self) -> Result<()> {
        self.child.write().await.kill().await?;
        Ok(())
    }

    pub fn get_stdout(&self) -> Option<broadcast::Receiver<Vec<u8>>> {
        self.stdout.as_ref().map(|x| x.resubscribe())
    }

    pub fn get_stderr(&self) -> Option<broadcast::Receiver<Vec<u8>>> {
        self.stderr.as_ref().map(|x| x.resubscribe())
    }

    pub fn get_stdin(&self) -> Option<mpsc::Sender<Vec<u8>>> {
        self.stdin.as_ref().map(|x| x.clone())
    }

    pub async fn state(&self) -> ProcessState {
        let mut child = self.child.write().await;

        match child.try_wait() {
            Err(e) => {
                warn!("Error while checking status of child process: {}", e);
                ProcessState::Dead
            }
            Ok(s) => {
                if s.is_none() {
                    ProcessState::Alive
                } else {
                    ProcessState::Dead
                }
            }
        }
    }
}

impl From<Process> for ProcessRef {
    fn from(value: Process) -> Self {
        Self::new(RwLock::new(value))
    }
}

pub type ProcessRef = Arc<RwLock<Process>>;

pub struct ProcessManagementService {
    processes: RwLock<HashMap<u64, ProcessRef>>,
}

impl ProcessManagementService {
    pub fn new() -> Self {
        Self {
            processes: RwLock::new(HashMap::new()),
        }
    }

    pub async fn new_process(
        &self,
        id: u64,
        launch_command: impl AsRef<OsStr>,
        arguments: impl IntoIterator<Item = impl AsRef<OsStr>>,
        work_dir: impl AsRef<Path>,
    ) -> Result<()> {
        let mut child = Command::new(launch_command);
        child
            .args(arguments)
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .stdin(Stdio::piped());

        let work_dir = work_dir.as_ref();
        if !work_dir.as_os_str().is_empty() {
            child.current_dir(work_dir);
        }

        let child = child.spawn()?;

        {
            let mut processes_write = self.processes.write().await;
            processes_write.insert(id, Process::new(child).into());
        }

        Ok(())
    }

    pub async fn get_process(&self, id: u64) -> Option<ProcessRef> {
        self.processes.read().await.get(&id).cloned()
    }
}
