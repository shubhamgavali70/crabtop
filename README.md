# Port Inspector

A production-ready Rust CLI that inspects the process currently listening on a specific TCP port. It reports the process name, PID, CPU usage (%), and memory usage (MB) with beautiful real-time visualizations. If `OPENAI_API_KEY` is set, it sends the stats to OpenAI and prints a brief insight.

## Features
- 🎯 **Port → PID resolution** using `lsof` (macOS/Linux) with Linux fallbacks (`ss`, `netstat`)
- 📊 **Real-time monitoring** with live dashboard and sparkline charts
- ⚡ **Accurate CPU tracking** via `sysinfo` with proper interval sampling
- 💾 **Memory usage visualization** with progress bars and history
- 🤖 **Optional OpenAI integration** (`gpt-4o-mini`) for AI-powered insights
- 🎨 **Beautiful terminal UI** with colors, progress bars, and sparklines
- 📈 **Historical data tracking** with averages and peak values
- ⌨️ **Interactive controls** - press 'q' or 'c' to quit watch mode

## Requirements
- `Rust` toolchain (1.70+ recommended; `sysinfo` requires 1.88 minimum per upstream docs).
- macOS or Linux.
  - macOS: `lsof` should be available by default.
  - Linux: Prefer `lsof`; otherwise `ss` (from `iproute2`) or `netstat` (from `net-tools`).
- Optional: `OPENAI_API_KEY` environment variable for AI insights.

## Build
```
cargo build --release
```
Binary will be at `target/release/port-inspector`.

## Usage

### Single Snapshot Mode
Get a one-time snapshot of process stats:
```bash
./target/release/port-inspector --port 8080
# or
./target/release/port-inspector -p 3000
```

Example output:
```
Process on port:
Name: my-service
PID: 12345
CPU: 3.42%
Memory: 128.53 MB
```

### Real-Time Monitoring Mode (Watch)
Monitor process stats in real-time with live visualizations:
```bash
./target/release/port-inspector --port 8080 --watch
# or with custom update interval (default: 1 second)
./target/release/port-inspector -p 8080 -w -i 2
```

The watch mode displays:
- 📊 Live CPU and memory usage with color-coded progress bars
- 📈 Sparkline charts showing historical trends
- 📉 Average and peak values
- ⏰ Real-time timestamp updates
- 🎨 Color-coded indicators (green/yellow/red based on usage)

Press `q` or `c` to exit watch mode.

### With OpenAI Insights
Set your OpenAI API key to get AI-powered insights:
```bash
export OPENAI_API_KEY=sk-...
./target/release/port-inspector -p 8080
```

Example output:
```
"my-service is running efficiently at 3.4% CPU with 128 MB memory usage, showing stable performance."
```

## How It Works

### PID Lookup
- Tries `lsof -n -P -iTCP:<port> -sTCP:LISTEN -t` first
- On Linux, falls back to parsing `ss -lntp`, then `netstat -lntp`

### Stats Collection
- Uses `sysinfo` with `System::new_all()` for proper CPU tracking initialization
- Refreshes the target process twice with 200ms interval using blocking thread sleep
- CPU calculation uses delta between two measurements for accuracy
- Memory reported as MB (decimal): `bytes / 1_000_000`

### Real-Time Monitoring
- Runs in a non-blocking loop with configurable update interval
- Maintains rolling history of last 60 samples for trend analysis
- Uses `crossterm` for terminal control and colored output
- Renders progress bars, sparklines, and statistics in real-time
- Graceful exit on 'q' or 'c' key press

### Visualization Components
- **Progress Bars**: Visual representation of current CPU/Memory usage
- **Sparklines**: Mini charts showing historical trends (▁▂▃▄▅▆▇█)
- **Color Coding**: 
  - Green: Normal usage
  - Yellow: Moderate usage
  - Red: High usage
- **Statistics**: Current, average, and peak values

### OpenAI Integration (Optional)
- Calls `https://api.openai.com/v1/chat/completions` with `gpt-4o-mini`
- Sends Name, PID, CPU, Memory for analysis
- Falls back to plain output on errors

## Cross-Platform Notes
- macOS/Linux prioritized and supported.
- Windows: Port→PID resolution is not implemented in this version.

## Troubleshooting
- "No process found listening on port X": Ensure the service is listening and `lsof/ss/netstat` are available.
- `lsof`/`ss` not found: Install the missing tool (`brew install lsof` on macOS if needed; `sudo apt install iproute2` or `net-tools` on Linux).
- OpenAI errors (invalid key, network issues): The tool prints the error and falls back to plain stats.

## Example Watch Mode Output

```
╔══════════════════════════════════════════════════════════════════════╗
║          PORT INSPECTOR - Real-time Monitoring (Port 8888)         ║
╚══════════════════════════════════════════════════════════════════════╝

📊 Process Information
   Name:      node
   PID:       12345
   Port:      8888
   Time:      2026-01-28 10:30:45
   Samples:   15

⚡ CPU Usage
   Current:    5.29%  [██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░]
   Average:    5.12%
   Peak:       6.84%
   History:   ▂▃▃▄▃▃▄▅▄▃▃▄▃▃▅

💾 Memory Usage
   Current:    42.07 MB  [████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░]
   Average:    41.23 MB
   Peak:       43.15 MB
   History:   ▄▄▅▅▅▆▆▆▆▇▇▇▇▇█

Press 'q' or 'c' to quit | Updates every second
```

## Command Line Options

```
Usage: port-inspector [OPTIONS] --port <PORT>

Options:
  -p, --port <PORT>          Target port to inspect
  -w, --watch                Enable real-time monitoring mode
  -i, --interval <INTERVAL>  Update interval in seconds for watch mode [default: 1]
  -h, --help                 Print help
```

## Development Notes
- Run in debug for quicker iteration: `cargo run -- -p 8080`
- Test watch mode: `cargo run -- -p 8080 --watch`
- The project uses `rustls` TLS in `reqwest` for portability
- Watch mode requires an interactive terminal (won't work in pipes or non-TTY contexts)

## Security
- The OpenAI API key is read from the environment at runtime and not stored.