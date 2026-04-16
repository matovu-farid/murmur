use crate::config::settings::config_dir;
use anyhow::{anyhow, Context, Result};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::fs;
use std::path::PathBuf;

fn pid_file_path() -> PathBuf {
    config_dir().join("murmur.pid")
}

pub fn write_pid_file() -> Result<()> {
    let path = pid_file_path();
    fs::create_dir_all(path.parent().unwrap()).ok();
    let pid = std::process::id();
    fs::write(&path, pid.to_string()).context("Failed to write PID file")?;
    Ok(())
}

pub fn read_pid_file() -> Option<i32> {
    let path = pid_file_path();
    let content = fs::read_to_string(&path).ok()?;
    content.trim().parse().ok()
}

pub fn remove_pid_file() -> Result<()> {
    let path = pid_file_path();
    if path.exists() {
        fs::remove_file(&path).context("Failed to remove PID file")?;
    }
    Ok(())
}

pub fn is_alive(pid: i32) -> bool {
    kill(Pid::from_raw(pid), None).is_ok()
}

pub fn stop_daemon() -> Result<()> {
    let pid = read_pid_file().ok_or_else(|| anyhow!("Daemon is not running (no PID file)"))?;
    if !is_alive(pid) {
        remove_pid_file()?;
        return Err(anyhow!("Daemon is not running (stale PID file removed)"));
    }
    kill(Pid::from_raw(pid), Signal::SIGTERM)
        .with_context(|| format!("Failed to send SIGTERM to PID {}", pid))?;
    remove_pid_file()?;
    Ok(())
}

pub fn spawn_background() -> Result<u32> {
    let exe = std::env::current_exe().context("Failed to get current executable path")?;
    let child = std::process::Command::new(&exe)
        .args(["start", "--foreground"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to spawn background process")?;
    Ok(child.id())
}

pub fn daemon_status() -> Option<(i32, u64)> {
    let pid = read_pid_file()?;
    if !is_alive(pid) {
        return None;
    }
    let uptime = pid_file_path()
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.elapsed().ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Some((pid, uptime))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_alive_for_current_process() {
        assert!(is_alive(std::process::id() as i32));
    }

    #[test]
    fn test_is_alive_for_invalid_pid() {
        assert!(!is_alive(999999));
    }
}
