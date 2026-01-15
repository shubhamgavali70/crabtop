mod cli;
mod gemini;
mod port;
mod process;

use clap::Parser;
use cli::Cli;
use port::find_pid_by_port;
use process::get_process_usage;
use std::{thread, time::Duration};

fn main() {
    let args = Cli::parse();

    let pid = match find_pid_by_port(args.port) {
        Ok(pid) => pid,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    if args.watch {
        watch_mode(args.port, pid);
    } else {
        single_run(args.port, pid);
    }
}

fn single_run(port: u16, pid: u32) {
    match get_process_usage(pid) {
        Ok(data) => print_usage(port, pid, data),
        Err(err) => eprintln!("Error: {}", err),
    }
}

fn watch_mode(port: u16, pid: u32) {
    println!("Watching port {} (PID {}) — press Ctrl+C to stop\n", port, pid);

    loop {
        match get_process_usage(pid) {
            Ok(data) => print_usage(port, pid, data),
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}

fn print_usage(port: u16, pid: u32, data: process::SystemProcessData) {
    // Format system data for AI analysis
    let system_info = format!(
        "Global CPU Usage: {:.2}%
Load Average: 1min={:.2}, 5min={:.2}, 15min={:.2}
Total Memory: {:.2} GB
Free Memory: {:.2} GB ({:.1}% free)
Total Swap: {:.2} GB
Free Swap: {:.2} GB ({:.1}% free)
CPU Cores: {}
Active Processes: {}",
        data.system.cpu_usage,
        data.system.load_avg_1,
        data.system.load_avg_5,
        data.system.load_avg_15,
        data.system.total_memory_gb,
        data.system.free_memory_gb,
        (data.system.free_memory_gb / data.system.total_memory_gb * 100.0),
        data.system.total_swap_gb,
        data.system.free_swap_gb,
        (data.system.free_swap_gb / data.system.total_swap_gb * 100.0),
        data.system.cpu_count,
        data.system.process_count
    );

    let process_info = format!(
        "Process Name: {}
PID: {}
CPU Usage: {:.2} cores ({:.1}%)
Memory Usage: {} MB ({:.2} GB)",
        data.process.name,
        pid,
        data.process.cpu_cores,
        data.process.cpu_cores * 100.0,
        data.process.memory_mb,
        data.process.memory_mb as f64 / 1024.0
    );

    // Try to get AI analysis
    match gemini::analyze_system_data(&system_info, &process_info, port, &data.process.name) {
        Ok(analysis) => {
            println!("{}", analysis);
            println!(); // Empty line for spacing
        }
        Err(err) => {
            // Fallback to basic output if AI fails
            eprintln!("⚠️  AI analysis unavailable: {}\n", err);
            println!("Port: {} | PID: {} | {}", port, pid, data.process.name);
            println!("CPU: {:.2} cores | Memory: {} MB\n", data.process.cpu_cores, data.process.memory_mb);
        }
    }
}
