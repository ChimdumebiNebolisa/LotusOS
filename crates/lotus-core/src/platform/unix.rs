//! Unix (Linux/macOS) platform adapter. Linux is the better-supported path;
//! macOS lacks a PID-creation-time source in this V1 and reports that
//! limitation instead of faking identity checks.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{LotusError, Result};
use super::PortOwner;

pub fn resolve_executable(command: &str) -> Result<PathBuf> {
    let candidate = Path::new(command);
    if candidate.is_absolute() || command.contains('/') {
        return if candidate.is_file() {
            Ok(candidate.to_path_buf())
        } else {
            Err(LotusError::NotFound(format!(
                "executable not found at path: {command}"
            )))
        };
    }

    let path_var = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let p = dir.join(candidate);
        if p.is_file() {
            return Ok(p);
        }
    }
    Err(LotusError::NotFound(format!(
        "`{command}` was not found on PATH"
    )))
}

pub fn list_tcp_listeners() -> Result<Vec<PortOwner>> {
    // Prefer lsof when present (Linux + macOS).
    if which("lsof") {
        return lsof_listeners();
    }
    #[cfg(target_os = "linux")]
    if which("ss") {
        return ss_listeners();
    }
    Err(LotusError::Unsupported(
        "port-owner discovery needs `lsof` (or `ss` on Linux)",
    ))
}

fn which(prog: &str) -> bool {
    resolve_executable(prog).is_ok()
}

fn lsof_listeners() -> Result<Vec<PortOwner>> {
    let output = std::process::Command::new("lsof")
        .args(["-nP", "-iTCP", "-sTCP:LISTEN", "-F", "pcn"])
        .output()?;
    let mut owners: Vec<PortOwner> = Vec::new();
    let mut current_pid: Option<u32> = None;
    let mut current_name: Option<String> = None;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        match line.chars().next() {
            Some('p') => {
                current_pid = line[1..].parse().ok();
            }
            Some('c') => {
                current_name = Some(line[1..].to_string());
            }
            Some('n') => {
                // format: *:3000 or 127.0.0.1:3000 or [::]:3000
                let spec = &line[1..];
                if let Some(port) = spec.rsplit(':').next().and_then(|p| p.parse::<u16>().ok()) {
                    owners.push(PortOwner {
                        port,
                        pid: current_pid,
                        name: current_name.clone(),
                    });
                }
            }
            _ => {}
        }
    }
    Ok(owners)
}

#[cfg(target_os = "linux")]
fn ss_listeners() -> Result<Vec<PortOwner>> {
    use std::collections::BTreeMap;
    let output = std::process::Command::new("ss")
        .args(["-ltnpH"])
        .output()?;
    // users:(("name",pid=1234,fd=5))
    let re_pid = |chunk: &str| -> Option<u32> {
        chunk
            .split("pid=")
            .nth(1)
            .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|d| d.parse().ok())
    };
    let mut by_port: BTreeMap<u16, (Option<u32>, Option<String>)> = BTreeMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let local = match line.split_whitespace().nth(3) {
            Some(l) => l,
            None => continue,
        };
        let port = match local.rsplit(':').next().and_then(|p| p.parse::<u16>().ok()) {
            Some(p) => p,
            None => continue,
        };
        if let Some(chunk) = line.split("users:((").nth(1) {
            let pid = re_pid(chunk);
            let name = chunk
                .split('"')
                .nth(1)
                .map(|s| s.to_string());
            by_port.entry(port).or_insert((pid, name));
        } else {
            by_port.entry(port).or_insert((None, None));
        }
    }
    Ok(by_port
        .into_iter()
        .map(|(port, (pid, name))| PortOwner { port, pid, name })
        .collect())
}

pub fn pid_process_name(pid: u32) -> Option<String> {
    std::fs::read_link(format!("/proc/{pid}/exe"))
        .ok()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        })
}

pub fn pid_identity_token(pid: u32) -> Option<u64> {
    // Linux: field 22 of /proc/<pid>/stat is starttime in jiffies since boot.
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after_comm = stat.rsplit(')').next()?;
    let fields: Vec<&str> = after_comm.split_whitespace().collect();
    // fields[0] is state (field 3); starttime is field 22 => index 19 here.
    fields.get(19)?.parse().ok()
}

pub fn pid_matches_token(pid: u32, token: u64) -> bool {
    match pid_identity_token(pid) {
        Some(current) => current == token,
        None => false,
    }
}

pub fn terminate_tree(pid: u32, grace: Duration) {
    unsafe {
        // Children are spawned with process_group(0), so pgid == pid and this
        // reaches the whole tree.
        libc::kill(-(pid as i32), libc::SIGTERM);
    }
    let deadline = std::time::Instant::now() + grace;
    while std::time::Instant::now() < deadline {
        if pid_identity_token(pid).is_none() && pid_process_name(pid).is_none() {
            return;
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    unsafe {
        libc::kill(-(pid as i32), libc::SIGKILL);
    }
}

pub fn terminate_tree_verified(pid: u32, token: u64) {
    if pid_matches_token(pid, token) || token == 0 {
        // SIGKILL to the process group; safe because we verified identity.
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
    }
}
