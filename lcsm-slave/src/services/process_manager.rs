use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    path::Path,
    process::Stdio,
    sync::Arc,
};

use anyhow::Result;

use tokio::{
    io::{AsyncRead, AsyncWrite},
    process::{Child, Command},
    sync::{RwLock, broadcast, mpsc},
};

use crate::transfer::{BinarySequence, redirect_input, redirect_output};

fn create_output_redirect(
    output: impl AsyncRead + Unpin + Sync + Send + 'static,
    child: Arc<RwLock<Child>>,
) -> broadcast::Receiver<BinarySequence> {
    let (tx, rx) = broadcast::channel(8);

    tokio::spawn(async move {
        if let Err(e) = redirect_output(output, tx, child).await {
            tracing::warn!("create output redirect: {}", e);
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
            tracing::warn!("create input redirect: {}", e);
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

impl ProcessState {
    pub async fn from_child(child: &Arc<RwLock<Child>>) -> Self {
        let mut child = child.write().await;

        match child.try_wait() {
            Err(e) => {
                tracing::warn!("Error while checking status of child process: {}", e);
                Self::Dead
            }
            Ok(s) => {
                if s.is_none() {
                    Self::Alive
                } else {
                    Self::Dead
                }
            }
        }
    }
}

impl Process {
    pub async fn setup(child: Child) -> Self {
        // create ref
        let child = Arc::new(RwLock::new(child));

        // stdio redirect
        let (stdout, stderr, stdin) = {
            let child_ref = &child;
            let mut child = child.write().await;

            let stdout = child
                .stdout
                .take()
                .map(|x| create_output_redirect(x, child_ref.clone()));
            let stderr = child
                .stderr
                .take()
                .map(|x| create_output_redirect(x, child_ref.clone()));
            let stdin = child.stdin.take().map(|x| create_input_redirect(x));

            (stdout, stderr, stdin)
        };

        Self {
            child,
            stdout,
            stderr,
            stdin,
        }
    }

    pub async fn kill(&self) -> Result<()> {
        let mut process = self.child.write().await;
        let exit_code = process.try_wait()?;
        if exit_code.is_none() {
            process.kill().await?;
        }

        Ok(())
    }

    pub fn get_stdout(&self) -> Option<broadcast::Receiver<BinarySequence>> {
        self.stdout.as_ref().map(|x| x.resubscribe())
    }

    pub fn get_stderr(&self) -> Option<broadcast::Receiver<BinarySequence>> {
        self.stderr.as_ref().map(|x| x.resubscribe())
    }

    pub fn get_stdin(&self) -> Option<mpsc::Sender<BinarySequence>> {
        self.stdin.as_ref().map(|x| x.clone())
    }

    pub async fn state(&self) -> ProcessState {
        ProcessState::from_child(&self.child).await
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

    fn generate_command_with_shell(
        launch_command: impl AsRef<OsStr>,
        arguments: impl IntoIterator<Item = impl AsRef<OsStr>>,
    ) -> Command {
        if cfg!(target_os = "macos") {
            // macOS
            let mut child = Command::new("script");

            // build launch arguments
            let mut arguments_vec: Vec<OsString> = vec![
                "-q".into(),
                "/dev/null".into(),
                "/bin/bash".into(),
                "-c".into(),
            ];

            arguments_vec.push(launch_command.as_ref().to_owned());
            for arg in arguments {
                arguments_vec.push(arg.as_ref().to_owned());
            }

            child
                .args(arguments_vec)
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .stdin(Stdio::piped());

            child
        } else {
            // unsupported
            tracing::warn!("Trying to spawn with shell on unsupported platform, fallback");
            Self::generate_command(launch_command, arguments)
        }
    }

    fn generate_command(
        launch_command: impl AsRef<OsStr>,
        arguments: impl IntoIterator<Item = impl AsRef<OsStr>>,
    ) -> Command {
        let mut child = Command::new(launch_command);
        child
            .args(arguments)
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .stdin(Stdio::piped());

        child
    }

    pub async fn new_process(
        &self,
        id: u64,
        launch_command: impl AsRef<OsStr>,
        arguments: impl IntoIterator<Item = impl AsRef<OsStr>>,
        work_dir: impl AsRef<Path>,
        use_shell: bool,
    ) -> Result<ProcessRef> {
        let mut child = if use_shell {
            Self::generate_command_with_shell(launch_command, arguments)
        } else {
            Self::generate_command(launch_command, arguments)
        };

        let work_dir = work_dir.as_ref();
        if !work_dir.as_os_str().is_empty() {
            child.current_dir(work_dir);
        }

        let child = child.spawn()?;

        // NOTICE: code about log_service is not here, you should go to `routes/...` to find it

        let process_ref = {
            let process_ref = ProcessRef::from(Process::setup(child).await);
            let mut processes_write = self.processes.write().await;
            processes_write.insert(id, process_ref.clone());
            process_ref
        };

        Ok(process_ref)
    }

    pub async fn get_process(&self, id: u64) -> Option<ProcessRef> {
        self.processes.read().await.get(&id).cloned()
    }
}
