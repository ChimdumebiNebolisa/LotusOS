//! Platform adapter boundary. All OS-specific behavior lives behind the
//! functions re-exported here so domain logic never branches on the
//! operating system.
//!
//! Contract for every implementation:
//! - `resolve_executable(command)` resolves through PATH (+PATHEXT on Windows).
//! - `pid_identity_token(pid)` returns a token stable for one process lifetime,
//!   used to defend against PID reuse before killing anything.
//! - `terminate_tree` never targets processes it has not verified.

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

/// A process currently listening on a local TCP port.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PortOwner {
    pub port: u16,
    pub pid: Option<u32>,
    pub name: Option<String>,
}
