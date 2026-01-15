use sysinfo::{Pid, System};
use std::{thread, time::Duration};

pub struct ProcessUsage {
    pub name: String,
    pub cpu_cores: f32,
    pub memory_mb: u64,
}

pub struct SystemSnapshot {
    pub cpu_usage: f32,
    pub load_avg_1: f64,
    pub load_avg_5: f64,
    pub load_avg_15: f64,
    pub total_memory_gb: f64,
    pub free_memory_gb: f64,
    pub total_swap_gb: f64,
    pub free_swap_gb: f64,
    pub cpu_count: usize,
    pub process_count: usize,
}

pub struct SystemProcessData {
    pub system: SystemSnapshot,
    pub process: ProcessUsage,
}

pub fn get_process_usage(pid: u32) -> Result<SystemProcessData, String> {
    let mut system = System::new_all();
    let pid = Pid::from(pid as usize);

    // Baseline - refresh twice with delay for accurate CPU measurement
    system.refresh_all();
    thread::sleep(Duration::from_millis(500));
    system.refresh_all();

    let process = system
        .process(pid)
        .ok_or("Process not found")?;

    let cpu_percent = process.cpu_usage();
    let cpu_cores = cpu_percent / 100.0;

    // Capture system snapshot
    let load_avg = System::load_average();
    let system_snapshot = SystemSnapshot {
        cpu_usage: system.global_cpu_info().cpu_usage(),
        load_avg_1: load_avg.one,
        load_avg_5: load_avg.five,
        load_avg_15: load_avg.fifteen,
        total_memory_gb: system.total_memory() as f64 / 1_073_741_824.0, // bytes to GB
        free_memory_gb: system.free_memory() as f64 / 1_073_741_824.0,
        total_swap_gb: system.total_swap() as f64 / 1_073_741_824.0,
        free_swap_gb: system.free_swap() as f64 / 1_073_741_824.0,
        cpu_count: system.cpus().len(),
        process_count: system.processes().len(),
    };

    let process_usage = ProcessUsage {
        name: process.name().to_string(),
        cpu_cores,
        memory_mb: process.memory() / 1024,
    };

    Ok(SystemProcessData {
        system: system_snapshot,
        process: process_usage,
    })
}
