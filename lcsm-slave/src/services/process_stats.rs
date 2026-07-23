use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug, Clone, Copy)]
pub struct ProcessMetrics {
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}

pub fn read_process_metrics(system: &mut System, pid: u32) -> Option<ProcessMetrics> {
    let pid = Pid::from_u32(pid);
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing().with_cpu().with_memory(),
    );

    let process = system.process(pid)?;
    Some(ProcessMetrics {
        cpu_percent: process.cpu_usage(),
        memory_bytes: process.memory(),
    })
}
