# Port Inspector - Usage Guide

## Quick Start

### Installation
```bash
# Build the project
cargo build --release

# The binary will be at:
# target/release/port-inspector
```

### Basic Usage

#### 1. Single Snapshot
Get a one-time snapshot of process statistics:

```bash
# Using full path
./target/release/port-inspector --port 8080

# Or add to PATH and use directly
port-inspector -p 3000
```

**Output:**
```
Process on port:
Name: node
PID: 12345
CPU: 5.29%
Memory: 42.07 MB
```

#### 2. Real-Time Monitoring (Watch Mode)
Monitor process in real-time with live visualizations:

```bash
# Watch with default 1-second updates
./target/release/port-inspector --port 8080 --watch

# Watch with custom interval (2 seconds)
./target/release/port-inspector -p 8080 -w -i 2
```

**Features in Watch Mode:**
- ✅ Live CPU and memory usage updates
- ✅ Color-coded progress bars (green/yellow/red)
- ✅ Sparkline charts showing trends
- ✅ Average and peak statistics
- ✅ Real-time timestamps
- ✅ Press 'q' or 'c' to exit

#### 3. With AI Insights (Optional)
Get AI-powered analysis of your process:

```bash
# Set your OpenAI API key
export OPENAI_API_KEY="sk-your-key-here"

# Run normally (single snapshot mode only)
./target/release/port-inspector -p 8080
```

**Output:**
```
"This node process is running efficiently at 5.3% CPU with 42 MB memory usage, showing stable performance."
```

## Common Use Cases

### Monitor a Web Server
```bash
# Monitor nginx on port 80
port-inspector -p 80 -w

# Monitor Node.js app on port 3000
port-inspector -p 3000 -w -i 2
```

### Check Database Performance
```bash
# PostgreSQL (default port 5432)
port-inspector -p 5432 -w

# MongoDB (default port 27017)
port-inspector -p 27017 -w
```

### Quick Health Check
```bash
# Single snapshot for quick check
port-inspector -p 8080
```

### Long-term Monitoring
```bash
# Watch with 5-second intervals for less frequent updates
port-inspector -p 8080 -w -i 5
```

## Tips & Tricks

### 1. Finding Which Port a Process Uses
```bash
# List all listening ports
lsof -iTCP -sTCP:LISTEN -P -n

# Then inspect the port
port-inspector -p <PORT> -w
```

### 2. Monitoring Multiple Processes
```bash
# Open multiple terminals and run:
port-inspector -p 3000 -w  # Terminal 1
port-inspector -p 8080 -w  # Terminal 2
port-inspector -p 5432 -w  # Terminal 3
```

### 3. Scripting and Automation
```bash
# Get stats in a script (single snapshot)
STATS=$(port-inspector -p 8080)
echo "$STATS"

# Watch mode requires interactive terminal
# Use single snapshot mode in scripts
```

### 4. Performance Tuning
```bash
# Use release build for better performance
cargo build --release

# The release build is optimized and much faster
./target/release/port-inspector -p 8080 -w
```

## Keyboard Controls (Watch Mode)

- `q` - Quit watch mode
- `c` - Quit watch mode (alternative)

## Color Indicators

### CPU Usage
- 🟢 **Green**: 0-50% (Normal)
- 🟡 **Yellow**: 50-80% (Moderate)
- 🔴 **Red**: 80-100% (High)

### Memory Usage
- 🔵 **Blue/Green**: 0-500 MB (Normal)
- 🟡 **Yellow**: 500-1000 MB (Moderate)
- 🔴 **Red**: 1000+ MB (High)

## Troubleshooting

### "No process found listening on port X"
- Ensure the service is actually running
- Check if `lsof` is installed: `which lsof`
- On Linux, try: `ss -lntp | grep <PORT>`

### "Failed to enable raw mode"
- Watch mode requires an interactive terminal
- Don't pipe output or redirect in watch mode
- Use single snapshot mode for scripting

### CPU shows 0.00%
- Process might be idle
- Try monitoring a process under load
- The tool waits 200ms between measurements

### OpenAI errors
- Check your API key: `echo $OPENAI_API_KEY`
- Ensure you have internet connectivity
- Tool falls back to plain output on errors

## Examples

### Example 1: Monitor Development Server
```bash
# Start your dev server
npm run dev  # Runs on port 3000

# In another terminal, monitor it
port-inspector -p 3000 -w
```

### Example 2: Check Production Performance
```bash
# Quick health check
port-inspector -p 8080

# Detailed monitoring
port-inspector -p 8080 -w -i 1
```

### Example 3: Debug High CPU
```bash
# Watch in real-time to see CPU spikes
port-inspector -p 8080 -w

# The sparkline will show historical trends
# Peak value shows maximum CPU usage
```

## Advanced Usage

### Building for Production
```bash
# Optimized release build
cargo build --release

# Install globally (optional)
cargo install --path .

# Now use from anywhere
port-inspector -p 8080 -w
```

### Cross-Platform Notes
- **macOS**: Fully supported, uses `lsof`
- **Linux**: Fully supported, uses `lsof`, `ss`, or `netstat`
- **Windows**: Port resolution not implemented

## Getting Help
```bash
# Show help message
port-inspector --help

# Show version
cargo pkgid
```
