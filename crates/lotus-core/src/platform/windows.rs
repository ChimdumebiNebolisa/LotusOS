//! Windows platform adapter.
#![allow(clippy::upper_case_acronyms)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{LotusError, Result};
use super::PortOwner;

const PATHEXT: [&str; 4] = [".COM", ".EXE", ".BAT", ".CMD"];

/// Run a subprocess with a hard deadline. Returns None on timeout/spawn
/// failure so callers treat it as "information unavailable" rather than
/// hanging the supervisor tick forever.
pub(crate) fn run_capped(
    program: &str,
    args: &[String],
    cap: Duration,
) -> Option<std::process::Output> {
    let mut command = std::process::Command::new(program);
    command
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(_) => return None,
    };
    let deadline = std::time::Instant::now() + cap;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Reap piped output without blocking indefinitely.
                use std::io::Read;
                let mut out: Vec<u8> = Vec::new();
                if let Some(mut s) = child.stdout.take() {
                    let _ = s.read_to_end(&mut out);
                }
                let mut err: Vec<u8> = Vec::new();
                if let Some(mut s) = child.stderr.take() {
                    let _ = s.read_to_end(&mut err);
                }
                return Some(std::process::Output {
                    status,
                    stdout: out,
                    stderr: err,
                });
            }
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(_) => return None,
        }
    }
}

const SUBPROC_CAP: Duration = Duration::from_secs(10);

pub fn resolve_executable(command: &str) -> Result<PathBuf> {
    let candidate = Path::new(command);
    let looks_pathed =
        candidate.is_absolute() || command.contains('\\') || command.contains('/');

    if looks_pathed {
        {
            let ext = "";
            let p = with_ext(candidate, ext);
            if p.is_file() {
                return Ok(p);
            }
        }
        // try adding extensions when no extension present
        if candidate.extension().is_none() {
            for ext in PATHEXT {
                let p = with_ext(candidate, ext);
                if p.is_file() {
                    return Ok(p);
                }
            }
        }
        return Err(LotusError::NotFound(format!(
            "executable not found at path: {command}"
        )));
    }

    let path_var = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let direct = dir.join(candidate);
        if direct.is_file() {
            return Ok(direct);
        }
        if candidate.extension().is_none() {
            for ext in PATHEXT {
                let p = with_ext(&direct, ext);
                if p.is_file() {
                    return Ok(p);
                }
            }
        }
    }
    Err(LotusError::NotFound(format!(
        "`{command}` was not found on PATH"
    )))
}

fn with_ext(path: &Path, ext: &str) -> PathBuf {
    if ext.is_empty() {
        path.to_path_buf()
    } else {
        PathBuf::from(format!("{}{ext}", path.display()))
    }
}

/// Parse `netstat -ano -p TCP` output for LISTENING rows.
pub fn list_tcp_listeners() -> Result<Vec<PortOwner>> {
    let output = run_capped("netstat", &[
        "-ano".to_string(), "-p".to_string(), "TCP".to_string()
    ], SUBPROC_CAP)
    .ok_or(LotusError::State("netstat unavailable or timed out".into()))?;
    if !output.status.success() {
        return Err(LotusError::State(format!(
            "netstat failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let mut by_port: BTreeMap<u16, u32> = BTreeMap::new();
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines().skip(4) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        // TCP <local> <remote> LISTENING <pid>
        if cols.len() != 5 || !cols[3].eq_ignore_ascii_case("LISTENING") {
            continue;
        }
        let local = cols[1];
        let port = match local.rsplit(':').next().and_then(|p| p.parse::<u16>().ok()) {
            Some(p) => p,
            None => continue,
        };
        if let Ok(pid) = cols[4].parse::<u32>() {
            by_port.entry(port).or_insert(pid);
        }
    }

    // One batched name table beats one tasklist spawn per PID
    // (~0.7s each => tens of seconds on real machines).
    let names = all_process_names();
    Ok(by_port
        .into_iter()
        .map(|(port, pid)| PortOwner {
            port,
            pid: Some(pid),
            name: names.get(&pid).cloned(),
        })
        .collect())
}

/// Single `tasklist` invocation -> pid -> image name.
fn all_process_names() -> BTreeMap<u32, String> {
    let mut map = BTreeMap::new();
    if let Some(output) = run_capped(
        "tasklist",
        &["/FO".to_string(), "CSV".to_string(), "/NH".to_string()],
        SUBPROC_CAP,
    ) {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            // "name.exe","1234","Console",...
            let mut parts = line.split("\",\"");
            if let (Some(name), Some(pid)) = (parts.next(), parts.next()) {
                if let Ok(pid) = pid.trim_matches('"').trim().parse::<u32>() {
                    let clean = name.trim_start_matches('"').to_string();
                    if !clean.is_empty() {
                        map.insert(pid, clean);
                    }
                }
            }
        }
    }
    map
}

pub fn pid_process_name(pid: u32) -> Option<String> {
    let output = run_capped(
        "tasklist",
        &[
            "/FI".to_string(),
            format!("PID eq {pid}"),
            "/FO".to_string(),
            "CSV".to_string(),
            "/NH".to_string(),
        ],
        SUBPROC_CAP,
    )?;
    let line = String::from_utf8_lossy(&output.stdout);
    let first = line.lines().find(|l| !l.trim().is_empty())?;
    let name = first.split(',').next()?.trim_matches('"');
    if name.eq_ignore_ascii_case("INFO:") || name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// Process creation time as Windows FILETIME (100ns ticks since 1601-01-01).
pub fn pid_identity_token(pid: u32) -> Option<u64> {
    use windows_sys::Win32::Foundation::{CloseHandle, FILETIME};
    use windows_sys::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut creation = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut exit = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut kernel = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut user = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let ok = GetProcessTimes(
            handle,
            &mut creation,
            &mut exit,
            &mut kernel,
            &mut user,
        );
        CloseHandle(handle);
        if ok != 0 && (creation.dwLowDateTime != 0 || creation.dwHighDateTime != 0) {
            Some(((creation.dwHighDateTime as u64) << 32) | creation.dwLowDateTime as u64)
        } else {
            None
        }
    }
}

pub fn pid_matches_token(pid: u32, token: u64) -> bool {
    pid_identity_token(pid) == Some(token)
}

fn taskkill(pid: u32, force: bool, tree: bool) {
    let mut args = vec!["/PID".to_string(), pid.to_string()];
    if tree {
        args.push("/T".to_string());
    }
    if force {
        args.push("/F".to_string());
    }
    let _ = run_capped("taskkill", &args, SUBPROC_CAP);
}

pub fn terminate_tree(pid: u32, grace: Duration) {
    // Best-effort graceful pass (only works for GUI apps), then wait, then force.
    taskkill(pid, false, true);
    let deadline = std::time::Instant::now() + grace;
    while std::time::Instant::now() < deadline {
        if pid_identity_token(pid).is_none() {
            return;
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    taskkill(pid, true, true);
}

pub fn terminate_tree_verified(pid: u32, token: u64) {
    if pid_matches_token(pid, token) {
        taskkill(pid, true, true);
    }
}
