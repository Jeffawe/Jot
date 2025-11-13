pub const PID_FILE: &str = "/tmp/jotx.pid";

pub fn is_running() -> bool {
    if let Ok(pid_str) = std::fs::read_to_string(PID_FILE) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            // Check if process exists
            return std::process::Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        }
    }
    false
}

pub fn save_pid() {
    let pid = std::process::id();
    let _ = std::fs::write(PID_FILE, pid.to_string());
}

pub fn remove_pid() {
    let _ = std::fs::remove_file(PID_FILE);
}