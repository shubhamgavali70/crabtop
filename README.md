# Port Inspector

A production-ready Rust CLI that inspects the process currently listening on a specific TCP port. It reports the process name, PID, CPU usage (%), and memory usage (MB). If `OPENAI_API_KEY` is set, it sends the stats to OpenAI and prints a brief, witty, beautified insight.

## Features
- Parse `--port`/`-p` via `clap` (u16).
- Port → PID resolution using `lsof` (macOS/Linux) with Linux fallbacks (`ss`, `netstat`).
- CPU and memory stats via `sysinfo` with proper CPU interval sampling.
- Optional OpenAI integration (`gpt-4o-mini` by default) via `reqwest` on `tokio`.
- Clear, human-readable output and graceful error handling.

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
Basic:
```
./target/release/port-inspector --port 8080
# or
./target/release/port-inspector -p 3000
```

Example output (no `OPENAI_API_KEY`):
```
Process on port:
Name: my-service
PID: 12345
CPU: 3.42%
Memory: 128.53 MB
```

With OpenAI insight:
```
export OPENAI_API_KEY=sk-...
./target/release/port-inspector -p 8080
"my-service is sipping CPU like a careful barista—steady at 3.4%. Memory’s cozy at 128 MB, not a byte wasted on drama."
```

## How It Works
- PID lookup:
  - Tries `lsof -n -P -iTCP:<port> -sTCP:LISTEN -t` first.
  - On Linux, falls back to parsing `ss -lntp`, then `netstat -lntp`.
- Stats collection:
  - Uses `sysinfo` and refreshes the target process twice, sleeping for `MINIMUM_CPU_UPDATE_INTERVAL` between refreshes to compute CPU usage over time.
  - Memory reported as MB (decimal): `bytes / 1_000_000`.
- OpenAI (optional):
  - Calls `https://api.openai.com/v1/chat/completions` with `gpt-4o-mini` by default.
  - Sends Name, PID, CPU, Memory; prints the first choice content.
  - Non-2xx responses are surfaced and the tool falls back to plain output.

## Cross-Platform Notes
- macOS/Linux prioritized and supported.
- Windows: Port→PID resolution is not implemented in this version.

## Troubleshooting
- "No process found listening on port X": Ensure the service is listening and `lsof/ss/netstat` are available.
- `lsof`/`ss` not found: Install the missing tool (`brew install lsof` on macOS if needed; `sudo apt install iproute2` or `net-tools` on Linux).
- OpenAI errors (invalid key, network issues): The tool prints the error and falls back to plain stats.

## Development Notes
- Run in debug for quicker iteration: `cargo run -- -p 8080`.
- The project uses `rustls` TLS in `reqwest` for portability.
- If you need JSON output, consider adding a `--json` flag (not implemented yet).

## Security
- The OpenAI API key is read from the environment at runtime and not stored.