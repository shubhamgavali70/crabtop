use std::env;
use std::process::Command;
use std::io::{self, Write};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use chrono::Local;
use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType, size as terminal_size},
};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::json;
use sysinfo::{Pid, System};

#[derive(Parser, Debug)]
#[command(name = "port-inspector", about = "Inspect the process listening on a given port.")]
struct Cli {
    /// Target port to inspect
    #[arg(short = 'p', long = "port")]
    port: u16,

    /// Enable real-time monitoring mode
    #[arg(short = 'w', long = "watch", default_value = "false")]
    watch: bool,

    /// Update interval in seconds for watch mode
    #[arg(short = 'i', long = "interval", default_value = "1")]
    interval: u64,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    name: String,
    pid: u32,
    cpu_percent: f32,
    memory_mb: f64,
}

struct ProcessHistory {
    cpu_history: Vec<f32>,
    mem_history: Vec<f64>,
    max_history: usize,
}

impl ProcessHistory {
    fn new(max_history: usize) -> Self {
        Self {
            cpu_history: Vec::new(),
            mem_history: Vec::new(),
            max_history,
        }
    }

    fn add(&mut self, info: &ProcessInfo) {
        self.cpu_history.push(info.cpu_percent);
        self.mem_history.push(info.memory_mb);

        if self.cpu_history.len() > self.max_history {
            self.cpu_history.remove(0);
        }
        if self.mem_history.len() > self.max_history {
            self.mem_history.remove(0);
        }
    }

    fn avg_cpu(&self) -> f32 {
        if self.cpu_history.is_empty() {
            0.0
        } else {
            self.cpu_history.iter().sum::<f32>() / self.cpu_history.len() as f32
        }
    }

    fn max_cpu(&self) -> f32 {
        self.cpu_history.iter().copied().fold(0.0f32, f32::max)
    }

    fn avg_mem(&self) -> f64 {
        if self.mem_history.is_empty() {
            0.0
        } else {
            self.mem_history.iter().sum::<f64>() / self.mem_history.len() as f64
        }
    }

    fn max_mem(&self) -> f64 {
        self.mem_history.iter().copied().fold(0.0f64, f64::max)
    }
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

    if cli.watch {
        // Real-time monitoring mode
        run_watch_mode(pid, cli.port, cli.interval).await?;
    } else {
        // Single snapshot mode
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
    }

    Ok(())
}

async fn run_watch_mode(pid: u32, port: u16, interval_secs: u64) -> Result<()> {
    let mut stdout = io::stdout();
    let mut history = ProcessHistory::new(60); // Keep last 60 samples
    let mut iteration = 0u64;
    let mut last_terminal_size = get_terminal_size();

    // Enable raw mode for better terminal control
    terminal::enable_raw_mode().context("Failed to enable raw mode")?;

    let result = async {
        loop {
            iteration += 1;

            // Check for terminal resize
            let current_size = get_terminal_size();
            if current_size != last_terminal_size {
                last_terminal_size = current_size;
                // Force redraw on resize
            }

            // Collect process info
            let info = match collect_process_info(pid).await {
                Ok(info) => info,
                Err(e) => {
                    terminal::disable_raw_mode()?;
                    return Err(e);
                }
            };

            history.add(&info);

            // Clear screen and move cursor to top
            execute!(
                stdout,
                terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            )?;

            // Render the dashboard with current terminal width
            render_dashboard(&mut stdout, &info, &history, port, iteration, last_terminal_size.0)?;

            stdout.flush()?;

            // Check for events (keyboard and resize) with non-blocking poll
            let mut should_break = false;
            let mut should_redraw = false;
            
            // Poll for events multiple times during the interval to be responsive
            let poll_duration = Duration::from_millis(100);
            let total_sleep = Duration::from_secs(interval_secs);
            let mut elapsed = Duration::ZERO;
            
            while elapsed < total_sleep {
                if event::poll(poll_duration)? {
                    match event::read()? {
                        Event::Key(key_event) => {
                            if key_event.code == KeyCode::Char('q')
                                || key_event.code == KeyCode::Char('c')
                                || key_event.code == KeyCode::Esc
                            {
                                should_break = true;
                                break;
                            }
                        }
                        Event::Resize(width, height) => {
                            last_terminal_size = (width, height);
                            should_redraw = true;
                            break; // Redraw immediately on resize
                        }
                        _ => {}
                    }
                }
                elapsed += poll_duration;
            }

            if should_break {
                break;
            }

            if should_redraw {
                continue; // Redraw immediately
            }
        }

        Ok::<(), anyhow::Error>(())
    }
    .await;

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(stdout, cursor::Show)?;

    result
}

fn get_terminal_size() -> (u16, u16) {
    terminal_size().unwrap_or((80, 24))
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

fn render_dashboard(
    stdout: &mut io::Stdout,
    info: &ProcessInfo,
    history: &ProcessHistory,
    port: u16,
    iteration: u64,
    terminal_width: u16,
) -> Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let width = terminal_width as usize;
    
    // Ensure minimum width
    let min_width = 60;
    let effective_width = width.max(min_width);
    
    // Calculate responsive sizes
    let bar_width = (effective_width.saturating_sub(30)).max(20).min(80);
    let sparkline_width = (effective_width.saturating_sub(20)).max(20).min(100);
    
    // Dynamic header
    let header_text = format!(" PORT INSPECTOR - Real-time Monitoring (Port {}) ", port);
    let header_text_len = header_text.len().min(effective_width.saturating_sub(2));
    let header_padding = effective_width.saturating_sub(header_text_len + 2);
    let left_pad = header_padding / 2;
    let right_pad = header_padding.saturating_sub(left_pad);
    let display_text = if header_text.len() > header_text_len {
        &header_text[..header_text_len]
    } else {
        &header_text
    };
    
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("╔"),
        Print("═".repeat(effective_width.saturating_sub(2))),
        Print("╗\n"),
        Print("║"),
        SetForegroundColor(Color::Yellow),
        Print(" ".repeat(left_pad)),
        Print(display_text),
        Print(" ".repeat(right_pad)),
        SetForegroundColor(Color::Cyan),
        Print("║\n"),
        Print("╚"),
        Print("═".repeat(effective_width.saturating_sub(2))),
        Print("╝\n"),
        ResetColor,
    )?;

    // Process Info
    execute!(
        stdout,
        Print("\n"),
        SetForegroundColor(Color::Green),
        Print("📊 Process Information\n"),
        ResetColor,
        Print(format!("   Name:      {}\n", info.name)),
        Print(format!("   PID:       {}\n", info.pid)),
        Print(format!("   Port:      {}\n", port)),
        Print(format!("   Time:      {}\n", timestamp)),
        Print(format!("   Samples:   {}\n", iteration)),
    )?;

    // CPU Section
    execute!(
        stdout,
        Print("\n"),
        SetForegroundColor(Color::Magenta),
        Print("⚡ CPU Usage\n"),
        ResetColor,
    )?;

    let cpu_color = if info.cpu_percent > 80.0 {
        Color::Red
    } else if info.cpu_percent > 50.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    execute!(
        stdout,
        Print("   Current:   "),
        SetForegroundColor(cpu_color),
        Print(format!("{:>6.2}%", info.cpu_percent)),
        ResetColor,
        Print("  "),
    )?;
    render_bar(stdout, info.cpu_percent as f64, 100.0, bar_width, cpu_color)?;
    execute!(stdout, Print("\n"))?;

    if !history.cpu_history.is_empty() {
        execute!(
            stdout,
            Print(format!("   Average:   {:>6.2}%\n", history.avg_cpu())),
            Print(format!("   Peak:      {:>6.2}%\n", history.max_cpu())),
        )?;

        // CPU Sparkline
        execute!(
            stdout,
            Print("   History:   "),
        )?;
        render_sparkline(stdout, &history.cpu_history, sparkline_width)?;
        execute!(stdout, Print("\n"))?;
    }

    // Memory Section
    execute!(
        stdout,
        Print("\n"),
        SetForegroundColor(Color::Blue),
        Print("💾 Memory Usage\n"),
        ResetColor,
    )?;

    let mem_color = if info.memory_mb > 1000.0 {
        Color::Red
    } else if info.memory_mb > 500.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let max_mem_display = if history.max_mem() > info.memory_mb {
        history.max_mem()
    } else {
        info.memory_mb
    };
    let mem_max = (max_mem_display * 1.2).max(100.0); // Add 20% headroom

    execute!(
        stdout,
        Print("   Current:   "),
        SetForegroundColor(mem_color),
        Print(format!("{:>8.2} MB", info.memory_mb)),
        ResetColor,
        Print("  "),
    )?;
    render_bar(stdout, info.memory_mb, mem_max, bar_width, mem_color)?;
    execute!(stdout, Print("\n"))?;

    if !history.mem_history.is_empty() {
        execute!(
            stdout,
            Print(format!("   Average:   {:>8.2} MB\n", history.avg_mem())),
            Print(format!("   Peak:      {:>8.2} MB\n", history.max_mem())),
        )?;

        // Memory Sparkline
        execute!(
            stdout,
            Print("   History:   "),
        )?;
        render_sparkline_mem(stdout, &history.mem_history, sparkline_width)?;
        execute!(stdout, Print("\n"))?;
    }

    // Footer
    execute!(
        stdout,
        Print("\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("Press 'q' or 'c' to quit | Updates every second\n"),
        ResetColor,
    )?;

    Ok(())
}

fn render_bar(
    stdout: &mut io::Stdout,
    value: f64,
    max: f64,
    width: usize,
    color: Color,
) -> Result<()> {
    let width = width.max(1); // Ensure at least 1 character width
    let filled = ((value / max.max(0.1)) * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width.saturating_sub(filled);

    execute!(
        stdout,
        Print("["),
        SetForegroundColor(color),
        Print("█".repeat(filled)),
        ResetColor,
        SetForegroundColor(Color::DarkGrey),
        Print("░".repeat(empty)),
        ResetColor,
        Print("]"),
    )?;

    Ok(())
}

fn render_sparkline(stdout: &mut io::Stdout, data: &[f32], width: usize) -> Result<()> {
    if data.is_empty() || width == 0 {
        return Ok(());
    }

    let sparkline_chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let max_val = data.iter().copied().fold(0.0f32, f32::max).max(1.0);

    let step = if data.len() > width {
        data.len() / width
    } else {
        1
    };

    let samples: Vec<f32> = data.iter().step_by(step).copied().collect();
    let display_samples = if samples.len() > width {
        &samples[samples.len() - width..]
    } else {
        &samples
    };

    for &val in display_samples {
        let normalized = (val / max_val).min(1.0);
        let idx = (normalized * (sparkline_chars.len() - 1) as f32).round() as usize;
        let color = if val > 80.0 {
            Color::Red
        } else if val > 50.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        execute!(
            stdout,
            SetForegroundColor(color),
            Print(sparkline_chars[idx]),
            ResetColor,
        )?;
    }

    Ok(())
}

fn render_sparkline_mem(stdout: &mut io::Stdout, data: &[f64], width: usize) -> Result<()> {
    if data.is_empty() || width == 0 {
        return Ok(());
    }

    let sparkline_chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let max_val = data.iter().copied().fold(0.0f64, f64::max).max(1.0);

    let step = if data.len() > width {
        data.len() / width
    } else {
        1
    };

    let samples: Vec<f64> = data.iter().step_by(step).copied().collect();
    let display_samples = if samples.len() > width {
        &samples[samples.len() - width..]
    } else {
        &samples
    };

    for &val in display_samples {
        let normalized = (val / max_val).min(1.0);
        let idx = (normalized * (sparkline_chars.len() - 1) as f64).round() as usize;
        let color = if val > 1000.0 {
            Color::Red
        } else if val > 500.0 {
            Color::Yellow
        } else {
            Color::Blue
        };
        execute!(
            stdout,
            SetForegroundColor(color),
            Print(sparkline_chars[idx]),
            ResetColor,
        )?;
    }

    Ok(())
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

    // Use tokio::task::spawn_blocking to run CPU measurement in a blocking context
    // This is necessary because sysinfo's CPU calculation works better with thread sleep
    let info = tokio::task::spawn_blocking(move || {
        let mut sys = System::new_all();
        
        // First refresh: Get baseline CPU measurement
        sys.refresh_process(pid);
        
        // Wait for at least 200ms to allow accurate CPU usage calculation
        // The sysinfo crate calculates CPU as a delta between two measurements
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        // Second refresh: Update to calculate CPU usage over the interval
        sys.refresh_process(pid);

        let proc = sys
            .process(pid)
            .ok_or_else(|| anyhow!("Failed to read process info for PID {}", pid.as_u32()))?;

        let name = proc.name().to_string();
        let cpu_percent = proc.cpu_usage();
        let memory_mb = (proc.memory() as f64) / 1_000_000.0; // bytes -> MB (decimal)

        Ok::<ProcessInfo, anyhow::Error>(ProcessInfo {
            name,
            pid: pid.as_u32(),
            cpu_percent,
            memory_mb,
        })
    })
    .await
    .context("Failed to spawn blocking task for process info collection")??;

    Ok(info)
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
        "Process Insight Request:\nName: {}\nPID: {}\nCPU: {:.2}%\nMemory: {:.2} MB\n\nPlease produce a brief insight about this process's resource consumption (1-2 sentences).",
        info.name, info.pid, info.cpu_percent, info.memory_mb
    );

    let body = json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "system", "content": "You are a tech genius performance analyst who summarizes resource usage succinctly."},
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