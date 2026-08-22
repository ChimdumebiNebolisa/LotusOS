//! Deterministic, bounded health checks.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::Path;
use std::time::Duration;

use crate::manifest::HealthSpec;

#[derive(Debug, Clone)]
pub struct Outcome {
    pub passed: bool,
    pub detail: String,
}

impl Outcome {
    fn pass(detail: impl Into<String>) -> Self {
        Outcome { passed: true, detail: detail.into() }
    }
    fn fail(detail: impl Into<String>) -> Self {
        Outcome { passed: false, detail: detail.into() }
    }
}

/// Evaluate every declared check in the spec. All checks must pass.
/// Every check is bounded by `spec.timeout_ms` (or less).
pub fn evaluate_all(spec: &HealthSpec, root: &Path) -> Vec<(&'static str, Outcome)> {
    let timeout = Duration::from_millis(spec.timeout_ms.min(60_000));
    let mut results = Vec::new();

    if let Some(port) = spec.port {
        match &spec.http_path {
            Some(path) => results.push(("http", http_check(port, path, spec.expect_status.unwrap_or(200), timeout))),
            None => results.push(("tcp", tcp_check(port, timeout))),
        }
    }
    if let Some(rel) = &spec.path {
        let abs = root.join(rel);
        results.push((
            "path",
            if abs.exists() {
                Outcome::pass(format!("{}", abs.display()))
            } else {
                Outcome::fail(format!("{} does not exist", abs.display()))
            },
        ));
    }
    if let Some(cmd) = &spec.command {
        results.push(("command", command_check(cmd, &spec.command_args, root, timeout)));
    }

    results
}

fn addr(port: u16) -> SocketAddr {
    SocketAddr::from((Ipv4Addr::LOCALHOST, port))
}

fn tcp_check(port: u16, timeout: Duration) -> Outcome {
    match TcpStream::connect_timeout(&addr(port), timeout) {
        Ok(_) => Outcome::pass(format!("127.0.0.1:{port} accepting connections")),
        Err(e) => Outcome::fail(format!("127.0.0.1:{port} not reachable: {e}")),
    }
}

/// Minimal local HTTP/1.0 GET over a raw socket. Plain HTTP only:
/// TLS and redirects are intentionally out of scope for V1 workspace
/// health checks (documented limitation).
fn http_check(port: u16, path: &str, expect_status: u16, timeout: Duration) -> Outcome {
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    let stream = match TcpStream::connect_timeout(&addr(port), timeout) {
        Ok(s) => s,
        Err(e) => return Outcome::fail(format!("connect failed: {e}")),
    };
    stream
        .set_read_timeout(Some(timeout))
        .and_then(|_| stream.set_write_timeout(Some(timeout)))
        .expect("socket timeouts");
    let mut stream = stream;
    let request = format!(
        "GET {path} HTTP/1.0\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    );
    if let Err(e) = stream.write_all(request.as_bytes()) {
        return Outcome::fail(format!("request write failed: {e}"));
    }
    let mut buf = [0u8; 512];
    let mut response = Vec::new();
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                response.extend_from_slice(&buf[..n]);
                if response.len() > 8192 {
                    break;
                }
            }
            Err(e) => return Outcome::fail(format!("read failed: {e}")),
        }
        if response.contains(&b'\n') {
            break;
        }
    }
    let text = String::from_utf8_lossy(&response);
    let status = text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok());
    match status {
        Some(code) if code == expect_status => Outcome::pass(format!("HTTP {code}")),
        Some(code) => Outcome::fail(format!("HTTP {code}, expected {expect_status}")),
        None => Outcome::fail(format!(
            "no parsable status line in response: {:.80}",
            text.trim()
        )),
    }
}

fn command_check(program: &str, args: &[String], cwd: &Path, timeout: Duration) -> Outcome {
    let resolved = match super::platform::resolve_executable(program) {
        Ok(p) => p,
        Err(e) => return Outcome::fail(format!("{e}")),
    };
    let mut child = match std::process::Command::new(resolved)
        .args(args)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return Outcome::fail(format!("spawn failed: {e}")),
    };
    let deadline = std::time::Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Outcome::pass("exit 0".to_string())
                } else {
                    Outcome::fail(format!("exit status: {status}"))
                };
            }
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Outcome::fail(format!("timed out after {}ms", timeout.as_millis()));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Outcome::fail(format!("wait failed: {e}")),
        }
    }
}
