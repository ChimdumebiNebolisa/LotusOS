//! Local append-only event ledger, one JSONL file per workspace.
//!
//! Events are the deterministic history of workspace lifecycle transitions.
//! Rotation: when a ledger exceeds `MAX_BYTES` it becomes `.jsonl.1`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::util::{format_ts, now_ms};

const MAX_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub seq: u64,
    pub ts_ms: u64,
    #[serde(skip_serializing)]
    pub ts_display: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub detail: serde_json::Value,
}

impl Event {
    pub fn new(kind: &str) -> Self {
        let ts = now_ms();
        Event {
            seq: 0,
            ts_ms: ts,
            ts_display: format_ts(ts),
            kind: kind.to_string(),
            process: None,
            detail: serde_json::Value::Null,
        }
    }

    pub fn for_process(kind: &str, process: &str) -> Self {
        let mut e = Event::new(kind);
        e.process = Some(process.to_string());
        e
    }

    pub fn with_detail(mut self, detail: serde_json::Value) -> Self {
        self.detail = detail;
        self
    }

    pub fn with_process(mut self, process: &str) -> Self {
        self.process = Some(process.to_string());
        self
    }
}

fn ledger_file(dir: &Path, key: &str) -> PathBuf {
    dir.join(key).join("events.jsonl")
}

/// Append an event, assigning the next sequence number.
pub fn append(dir: &Path, key: &str, mut event: Event) -> Result<Event> {
    let path = ledger_file(dir, key);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if path.exists() && std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) >= MAX_BYTES {
        let backup = path.with_extension("jsonl.1");
        let _ = std::fs::remove_file(&backup);
        let _ = std::fs::rename(&path, backup);
    }
    event.seq = next_seq(&path);
    let mut line = serde_json::to_string(&event)?;
    line.push('\n');
    use std::io::Write;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?
        .write_all(line.as_bytes())?;
    Ok(event)
}

fn next_seq(path: &Path) -> u64 {
    read_all(path).last().map(|e| e.seq + 1).unwrap_or(1)
}

pub fn read_all(path: &Path) -> Vec<Event> {
    match std::fs::read(path) {
        Ok(bytes) => String::from_utf8_lossy(&bytes)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                serde_json::from_str::<RawEvent>(l)
                    .ok()
                    .map(|raw| raw.into_event())
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Last `limit` events (oldest first).
pub fn tail(dir: &Path, key: &str, limit: usize) -> Vec<Event> {
    let all = read_all(&ledger_file(dir, key));
    let start = all.len().saturating_sub(limit);
    all[start..].to_vec()
}

#[derive(Deserialize)]
struct RawEvent {
    seq: u64,
    ts_ms: u64,
    kind: String,
    #[serde(default)]
    process: Option<String>,
    #[serde(default)]
    detail: serde_json::Value,
}

impl RawEvent {
    fn into_event(self) -> Event {
        Event {
            seq: self.seq,
            ts_ms: self.ts_ms,
            ts_display: format_ts(self.ts_ms),
            kind: self.kind,
            process: self.process,
            detail: self.detail,
        }
    }
}
