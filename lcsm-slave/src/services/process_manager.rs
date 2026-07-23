use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    path::Path,
    process::Stdio,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use sysinfo::System;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    process::{Child, Command},
    sync::{Mutex, RwLock, broadcast, mpsc},
};

use crate::{
    services::process_stats::{ProcessMetrics, read_process_metrics},
    transfer::{BinarySequence, redirect_input, redirect_output},
};

fn shell_escape(input: &OsStr) -> String {
    let text = input.to_string_lossy();
    let mut escaped = String::from("'");
    for ch in text.chars() {
        if ch == '\'' {
            escaped.push_str("'\"'\"'");
        } else {
            escaped.push(ch);
        }
    }
    escaped.push('\'');
    escaped
}

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

    rx
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

    tx
}

pub struct Process {
    child: Arc<RwLock<Child>>,
    pid: Option<u32>,
    started_at: SystemTime,

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessRuntimeStatusKind {
    Running,
    Stopped,
}

impl ProcessRuntimeStatusKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessRuntimeStatus {
    pub status: ProcessRuntimeStatusKind,
    pub pid: Option<u32>,
    pub started_at: Option<u64>,
    pub uptime_ms: Option<u64>,
    pub cpu_percent: Option<f32>,
    pub memory_bytes: Option<u64>,
}

impl ProcessRuntimeStatus {
    pub fn stopped() -> Self {
        Self {
            status: ProcessRuntimeStatusKind::Stopped,
            pid: None,
            started_at: None,
            uptime_ms: None,
            cpu_percent: None,
            memory_bytes: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.status == ProcessRuntimeStatusKind::Running
    }
}

impl Process {
    pub async fn setup(child: Child, started_at: SystemTime) -> Self {
        let pid = child.id();

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
            pid,
            started_at,
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

    pub fn pid(&self) -> Option<u32> {
        self.pid
    }

    pub async fn state(&self) -> ProcessState {
        ProcessState::from_child(&self.child).await
    }

    pub fn snapshot(&self, metrics: Option<ProcessMetrics>) -> ProcessRuntimeStatus {
        let started_at = self
            .started_at
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_secs());
        let uptime_ms = SystemTime::now()
            .duration_since(self.started_at)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_millis()).ok());

        ProcessRuntimeStatus {
            status: ProcessRuntimeStatusKind::Running,
            pid: self.pid,
            started_at,
            uptime_ms,
            cpu_percent: metrics.map(|value| value.cpu_percent),
            memory_bytes: metrics.map(|value| value.memory_bytes),
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
    stats_system: Mutex<System>,
}

impl ProcessManagementService {
    pub fn new() -> Self {
        Self {
            processes: RwLock::new(HashMap::new()),
            stats_system: Mutex::new(System::new()),
        }
    }

    fn generate_command_with_shell(
        launch_command: impl AsRef<OsStr>,
        arguments: impl IntoIterator<Item = impl AsRef<OsStr>>,
    ) -> Command {
        let launch_command = launch_command.as_ref().to_owned();
        let arguments = arguments
            .into_iter()
            .map(|arg| arg.as_ref().to_owned())
            .collect::<Vec<OsString>>();
        let shell_command = std::iter::once(shell_escape(&launch_command))
            .chain(arguments.iter().map(|arg| shell_escape(arg)))
            .collect::<Vec<_>>()
            .join(" ");

        if cfg!(target_os = "macos") {
            let mut child = Command::new("script");
            child
                .args(["-q", "/dev/null", "/bin/bash", "-c", &shell_command])
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .stdin(Stdio::piped());
            child
        } else if cfg!(target_os = "linux") {
            let mut child = Command::new("/bin/bash");
            child
                .args(["-lc", &shell_command])
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .stdin(Stdio::piped());
            child
        } else {
            tracing::warn!("Trying to spawn with shell on unsupported platform, fallback");
            Self::generate_command(&launch_command, &arguments)
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
        let started_at = SystemTime::now();

        // NOTICE: code about log_service is not here, you should go to `routes/...` to find it

        let process_ref = {
            let process_ref = ProcessRef::from(Process::setup(child, started_at).await);
            let mut processes_write = self.processes.write().await;
            processes_write.insert(id, process_ref.clone());
            process_ref
        };

        if let Some(pid) = process_ref.read().await.pid() {
            let mut system = self.stats_system.lock().await;
            let _ = read_process_metrics(&mut system, pid);
        }

        Ok(process_ref)
    }

    pub async fn get_process(&self, id: u64) -> Option<ProcessRef> {
        self.processes.read().await.get(&id).cloned()
    }

    pub async fn get_runtime_status(&self, id: u64) -> ProcessRuntimeStatus {
        let Some(process) = self.get_process(id).await else {
            return ProcessRuntimeStatus::stopped();
        };

        let process = process.read().await;
        if process.state().await == ProcessState::Dead {
            return ProcessRuntimeStatus::stopped();
        }

        let metrics = if let Some(pid) = process.pid() {
            let mut system = self.stats_system.lock().await;
            read_process_metrics(&mut system, pid)
        } else {
            None
        };

        process.snapshot(metrics)
    }
}
