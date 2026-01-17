use std::env;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::json;
use sysinfo::{Pid, System, MINIMUM_CPU_UPDATE_INTERVAL};

#[derive(Parser, Debug)]
#[command(name = "port-inspector", about = "Inspect the process listening on a given port.")]
struct Cli {
    /// Target port to inspect
    #[arg(short = 'p', long = "port")]
    port: u16,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    name: String,
    pid: u32,
    cpu_percent: f32,
    memory_mb: f64,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    let pid = find_pid_by_port(cli.port)
        .with_context(|| format!("No process found listening on port {}", cli.port))?;

    let info = collect_process_info(pid).await?;

    match env::var("OPENAI_API_KEY") {
        Ok(api_key) if !api_key.trim().is_empty() => {
            match generate_openai_insight(&api_key, &info).await {
                Ok(text) => {
                    println!("{}", text);
                }
                Err(err) => {
                    eprintln!("OpenAI call failed: {}", err);
                    print_plain(&info);
                }
            }
        }
        _ => {
            print_plain(&info);
        }
    }

    Ok(())
}

fn print_plain(info: &ProcessInfo) {
    println!(
        "Process on port:\nName: {name}\nPID: {pid}\nCPU: {cpu:.2}%\nMemory: {mem:.2} MB",
        name = info.name,
        pid = info.pid,
        cpu = info.cpu_percent,
        mem = info.memory_mb
    );
}

// Tries to resolve the PID listening on the given port using lsof first,
// then platform-specific fallbacks on Linux.
fn find_pid_by_port(port: u16) -> Result<u32> {
    // Prefer lsof (works well on macOS and most Linux distros)
    // lsof flags:
    // -n: no DNS
    // -P: no port service name translation
    // -iTCP:<port>: filter TCP for specific port
    // -sTCP:LISTEN: only listening sockets
    // -t: terse output (just PIDs)
    let lsof_args = [
        "-n",
        "-P",
        &format!("-iTCP:{}", port),
        "-sTCP:LISTEN",
        "-t",
    ];

    let lsof_out = Command::new("lsof")
        .args(&lsof_args)
        .output();

    if let Ok(out) = lsof_out {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = stdout.lines().find(|l| !l.trim().is_empty()) {
                let pid: u32 = line.trim().parse().context("Failed to parse PID from lsof")?;
                return Ok(pid);
            }
        }
    }

    // Fallbacks for Linux: try `ss -lntp`
    #[cfg(target_os = "linux")]
    {
        let ss_out = Command::new("ss")
            .args(["-lntp"]) // listening, numeric, tcp, show process
            .output();

        if let Ok(out) = ss_out {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Example line:
                // LISTEN 0 128 0.0.0.0:80 ... users:(("nginx",pid=1234,fd=7))
                for line in stdout.lines() {
                    if line.contains(&format!(":{} ", port)) || line.ends_with(&format!(":{}", port)) {
                        if let Some(pid_str) = line.split("pid=").nth(1) {
                            let pid_part = pid_str.split(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
                            if !pid_part.is_empty() {
                                let pid: u32 = pid_part.parse().context("Failed to parse PID from ss output")?;
                                return Ok(pid);
                            }
                        }
                    }
                }
            }
        }

        // Try netstat as a last resort (may require `net-tools`)
        let netstat_out = Command::new("netstat")
            .args(["-lntp"]).output();
        if let Ok(out) = netstat_out {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Typical line contains "0.0.0.0:<port>" and "pid/program"
                for line in stdout.lines() {
                    if line.contains(&format!(":{}", port)) {
                        // Extract pid from the last column like "1234/program"
                        if let Some(last_col) = line.split_whitespace().last() {
                            if let Some(pid_part) = last_col.split('/').next() {
                                if let Ok(pid) = pid_part.parse::<u32>() {
                                    return Ok(pid);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, lsof is the practical way; if it failed, surface error.
        return Err(anyhow!(
            "Failed to resolve PID on port {}. Ensure `lsof` is installed and accessible.",
            port
        ));
    }

    #[cfg(target_os = "windows")]
    {
        return Err(anyhow!(
            "Port-to-PID resolution is not implemented on Windows in this tool.",
        ));
    }

    // Other platforms
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(anyhow!("Unsupported OS for port inspection."))
    }
}

async fn collect_process_info(pid: u32) -> Result<ProcessInfo> {
    let pid = Pid::from_u32(pid);

    let mut sys = System::new();
    // First refresh to build baseline
    sys.refresh_process(pid);
    tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL).await;
    // Second refresh to compute CPU over interval
    sys.refresh_process(pid);

    let proc = sys
        .process(pid)
        .ok_or_else(|| anyhow!("Failed to read process info for PID {}", pid.as_u32()))?;

    let name = proc.name().to_string();
    let cpu_percent = proc.cpu_usage();
    let memory_mb = (proc.memory() as f64) / 1_000_000.0; // bytes -> MB (decimal)

    Ok(ProcessInfo {
        name,
        pid: pid.as_u32(),
        cpu_percent,
        memory_mb,
    })
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Deserialize)]
struct OpenAIMessage {
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

async fn generate_openai_insight(api_key: &str, info: &ProcessInfo) -> Result<String> {
    let client = reqwest::Client::new();

    let prompt = format!(
        "Process Insight Request:\nName: {}\nPID: {}\nCPU: {:.2}%\nMemory: {:.2} MB\n\nPlease produce a brief, witty, and beautified insight about this process's resource consumption (1-3 sentences).",
        info.name, info.pid, info.cpu_percent, info.memory_mb
    );

    let body = json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "system", "content": "You are a witty performance analyst who summarizes resource usage succinctly."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.7
    });

    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .context("Failed to call OpenAI API")?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(anyhow!("OpenAI API error: {} - {}", status, text));
    }

    let parsed: OpenAIResponse = res.json().await.context("Failed to parse OpenAI response")?;
    let content = parsed
        .choices
        .get(0)
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow!("No choices returned by OpenAI"))?;

    Ok(content)
}