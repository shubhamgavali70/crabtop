use std::process::Command;

pub fn find_pid_by_port(port: u16) -> Result<u32, String> {
    let output = Command::new("lsof")
        .arg("-i")
        .arg(format!(":{}", port))
        .arg("-sTCP:LISTEN")
        .arg("-t")
        .output()
        .map_err(|e| format!("Failed to execute lsof: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "lsof command failed. Is anything listening on port {}?",
            port
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let pid_str = stdout
        .lines()
        .next()
        .ok_or("No process found for this port")?;

    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|_| "Failed to parse PID")?;

    Ok(pid)
}
