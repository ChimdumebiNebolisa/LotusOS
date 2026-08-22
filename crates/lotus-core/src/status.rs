//! Workspace status and control-plane file formats.
//!
//! The supervisor writes `status.json` as a heartbeat every tick; readers
//! (CLI, Tauri) treat it as stale when `updated_at_ms` falls too far behind,
//! which is how a crashed LotusOS process is detected while workspace
//! processes survive.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::util::now_ms;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceState {
    Off,
    Starting,
    Healthy,
    Degraded,
    Failed,
    Stopping,
}

impl WorkspaceState {
    pub fn label(&self) -> &'static str {
        match self {
            WorkspaceState::Off => "OFF",
            WorkspaceState::Starting => "STARTING",
            WorkspaceState::Healthy => "HEALTHY",
            WorkspaceState::Degraded => "DEGRADED",
            WorkspaceState::Failed => "FAILED",
            WorkspaceState::Stopping => "STOPPING",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessState {
    Pending,
    Starting,
    Running,
    Unhealthy,
    Restarting,
    Exited,
    Crashed,
    Failed,
    Stopping,
    Stopped,
}

impl ProcessState {
    pub fn is_terminal_bad(&self) -> bool {
        matches!(self, ProcessState::Failed)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessStatus {
    pub name: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    /// Platform identity token captured at spawn; used for safe orphan kills.
    #[serde(default)]
    pub identity_token: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthy: Option<bool>,
    #[serde(default)]
    pub restarts: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusReport {
    pub key: String,
    pub name: String,
    pub root: String,
    pub manifest_hash: String,
    pub state: String,
    pub started_at_ms: Option<u64>,
    pub updated_at_ms: u64,
    pub processes: Vec<ProcessStatus>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub port_conflicts: Vec<crate::ports::PortConflict>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

/// How long before a status heartbeat counts as stale (supervisor crashed).
pub const STALE_AFTER_MS: u64 = 6000;

impl StatusReport {
    pub fn fresh(&self) -> bool {
        now_ms().saturating_sub(self.updated_at_ms) < STALE_AFTER_MS
    }

    /// Live PIDs recorded by the last heartbeat (used for orphan cleanup).
    pub fn live_pids(&self) -> Vec<(u32, u64)> {
        self.processes
            .iter()
            .filter_map(|p| p.pid.map(|pid| (pid, p.identity_token)))
            .collect()
    }
}

pub fn read(path: &Path) -> Option<StatusReport> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlRequest {
    pub action: ControlAction,
    pub requested_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlAction {
    Stop,
}

pub fn request_stop(control_path: &Path) -> std::io::Result<()> {
    if let Some(parent) = control_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let req = ControlRequest {
        action: ControlAction::Stop,
        requested_at_ms: now_ms(),
    };
    std::fs::write(control_path, serde_json::to_vec(&req)?)
}

pub fn read_control(control_path: &Path) -> Option<ControlRequest> {
    let bytes = std::fs::read(control_path).ok()?;
    serde_json::from_slice(&bytes).ok()
}
