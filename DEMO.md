# Port Inspector - Live Demo

## 🎬 Quick Demo

### Step 1: Single Snapshot Mode
```bash
$ port-inspector -p 8080

Process on port:
Name: node
PID: 12345
CPU: 5.29%
Memory: 42.07 MB
```
**Use Case:** Quick health check, scripting, automation

---

### Step 2: Real-Time Monitoring Mode
```bash
$ port-inspector -p 8080 --watch
```

**Output (updates every second):**
```
╔══════════════════════════════════════════════════════════════════════╗
║          PORT INSPECTOR - Real-time Monitoring (Port 8080)         ║
╚══════════════════════════════════════════════════════════════════════╝

📊 Process Information
   Name:      node
   PID:       12345
   Port:      8080
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

**Features Shown:**
- ✅ Live updating statistics
- ✅ Color-coded progress bars
- ✅ Sparkline trend charts
- ✅ Average and peak tracking
- ✅ Real-time timestamps
- ✅ Sample counter

**Use Case:** Performance monitoring, debugging, optimization

---

## 🎨 Visual Elements Explained

### Progress Bars
```
[████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░]
 ^^^^^^^^                              
 Filled portion = current usage
         ░░░░░░░░░░░░░░░░░░░░░░░░░░░░
         Empty portion = available
```

### Sparklines
```
▁▂▃▄▅▆▇█
│││││││└─ Highest point
││││││└── Very high
│││││└─── High
││││└──── Above average
│││└───── Average
││└────── Below average
│└─────── Low
└──────── Lowest point
```

### Color Coding

#### CPU Colors
- 🟢 **Green** (0-50%): `[██░░░░░░░░]` Normal operation
- 🟡 **Yellow** (50-80%): `[█████░░░░░]` Moderate load
- 🔴 **Red** (80-100%): `[█████████░]` High load

#### Memory Colors
- 🔵 **Blue/Green** (0-500 MB): Normal
- 🟡 **Yellow** (500-1000 MB): Moderate
- 🔴 **Red** (1000+ MB): High

---

## 🎯 Real-World Scenarios

### Scenario 1: Monitoring a Web Server
```bash
# Start monitoring your web server
$ port-inspector -p 80 --watch

# What you'll see:
# - CPU spikes during request handling
# - Memory growth with active connections
# - Trends showing traffic patterns
# - Peak values during high load
```

### Scenario 2: Debugging Memory Leaks
```bash
# Monitor over time
$ port-inspector -p 3000 --watch

# Look for:
# - Steadily increasing memory in sparkline: ▁▂▃▄▅▆▇█
# - Growing average memory value
# - Peak memory continuously rising
# - No memory drops (no garbage collection)
```

### Scenario 3: Performance Optimization
```bash
# Before optimization
$ port-inspector -p 8080 --watch
CPU: 45.2%  [████████████░░░░░░░░░░]  ← High CPU

# After optimization
$ port-inspector -p 8080 --watch
CPU: 12.5%  [███░░░░░░░░░░░░░░░░░░░]  ← Much better!
```

### Scenario 4: Load Testing
```bash
# Monitor during load test
$ port-inspector -p 8080 -w -i 1

# Observe:
# - CPU usage patterns
# - Memory consumption
# - Peak values under load
# - Recovery after load
```

---

## 🎮 Interactive Demo

### Try It Yourself!

1. **Find a running process:**
   ```bash
   lsof -iTCP -sTCP:LISTEN -P -n | grep LISTEN
   ```

2. **Quick check:**
   ```bash
   port-inspector -p <PORT>
   ```

3. **Watch mode:**
   ```bash
   port-inspector -p <PORT> --watch
   ```

4. **Generate some load:**
   ```bash
   # In another terminal, make requests
   while true; do curl http://localhost:<PORT>; sleep 0.1; done
   ```

5. **Observe the changes:**
   - CPU usage increases
   - Sparkline shows activity
   - Peak values update
   - Colors change based on load

---

## 📊 Sample Data Interpretation

### Example 1: Healthy Service
```
CPU:     2.5%   [█░░░░░░░░░░░░░░░░░░░]  ← Low, stable
Average: 2.3%
Peak:    3.1%
History: ▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂              ← Flat line = stable

Memory:  45 MB  [██░░░░░░░░░░░░░░░░░░]  ← Reasonable
Average: 44 MB
Peak:    47 MB
History: ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄              ← Stable memory
```
**Interpretation:** Service is healthy, stable, and efficient.

### Example 2: Under Load
```
CPU:     78.4%  [███████████████████░]  ← High usage
Average: 65.2%
Peak:    92.1%
History: ▃▄▅▆▇█▇▆▅▄▃▄▅▆▇              ← Fluctuating

Memory:  512 MB [████████████░░░░░░░░]  ← Growing
Average: 445 MB
Peak:    512 MB
History: ▄▅▅▆▆▇▇▇███                  ← Trending up
```
**Interpretation:** Service under heavy load, consider scaling.

### Example 3: Memory Leak
```
CPU:     5.2%   [█░░░░░░░░░░░░░░░░░░░]  ← Normal
Average: 5.1%
Peak:    6.3%
History: ▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂              ← Stable

Memory:  1.2 GB [████████████████████]  ← Very high!
Average: 980 MB
Peak:    1.2 GB
History: ▁▂▃▄▅▆▇████████              ← Continuously growing!
```
**Interpretation:** Possible memory leak! Memory keeps growing.

---

## 🎓 Tips for Effective Monitoring

### 1. Baseline Measurement
```bash
# Establish baseline when idle
port-inspector -p 8080 --watch

# Note the "normal" values:
# - Idle CPU: ~1-2%
# - Idle Memory: ~50 MB
```

### 2. Compare Before/After
```bash
# Before changes
port-inspector -p 8080 -w  # Note the values

# Make changes, restart service

# After changes
port-inspector -p 8080 -w  # Compare
```

### 3. Long-term Monitoring
```bash
# Use longer intervals for sustained monitoring
port-inspector -p 8080 -w -i 5

# Less frequent updates = less noise
# Easier to spot long-term trends
```

### 4. Multiple Processes
```bash
# Open multiple terminals
# Terminal 1: Frontend
port-inspector -p 3000 -w

# Terminal 2: Backend
port-inspector -p 8080 -w

# Terminal 3: Database
port-inspector -p 5432 -w
```

---

## 🚀 Advanced Usage

### Custom Intervals
```bash
# Fast updates (0.5 seconds) - not recommended, use 1 second minimum
port-inspector -p 8080 -w -i 1

# Slow updates (10 seconds)
port-inspector -p 8080 -w -i 10
```

### Scripting Integration
```bash
# Single snapshot for scripts
STATS=$(port-inspector -p 8080)
echo "$STATS"

# Parse output
CPU=$(echo "$STATS" | grep "CPU:" | awk '{print $2}')
echo "CPU Usage: $CPU"
```

---

## 🎉 Summary

Port Inspector provides:
- ✅ **Instant feedback** with single snapshot mode
- ✅ **Real-time monitoring** with watch mode
- ✅ **Visual trends** with sparklines
- ✅ **Statistical analysis** with averages and peaks
- ✅ **Color-coded alerts** for quick assessment
- ✅ **Historical tracking** for pattern detection

**Perfect for:**
- Development and debugging
- Performance optimization
- Load testing
- Production monitoring
- Capacity planning
- Troubleshooting

**Try it now:**
```bash
port-inspector -p <YOUR_PORT> --watch
```
