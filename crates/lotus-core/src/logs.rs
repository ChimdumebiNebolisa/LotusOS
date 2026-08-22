//! Bounded, rotating process log files.
//!
//! Layout: `<data>/logs/<workspace_key>/<process>.out.log` and `.err.log`.
//! Rotation: when a file exceeds `max_bytes` it is renamed to `.log.1`
//! (previous `.1` is overwritten). Two generations max keeps growth bounded.

use std::io::Write;
use std::path::{Path, PathBuf};

const DEFAULT_MAX_BYTES: u64 = 5 * 1024 * 1024;

pub struct LogWriter {
    path: PathBuf,
    file: Option<std::fs::File>,
    written: u64,
    max_bytes: u64,
}

impl LogWriter {
    pub fn open(path: PathBuf) -> std::io::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        let written = file.metadata().map(|m| m.len()).unwrap_or(0);
        Ok(LogWriter {
            path,
            file: Some(file),
            written,
            max_bytes: DEFAULT_MAX_BYTES,
        })
    }

    pub fn write_line(&mut self, line: &[u8]) {
        if self.written >= self.max_bytes {
            self.rotate();
        }
        let Some(file) = self.file.as_mut() else { return };
        let _ = file.write_all(line);
        self.written += line.len() as u64;
    }

    fn rotate(&mut self) {
        self.file = None;
        let backup = sibling(&self.path, ".1");
        let _ = std::fs::remove_file(&backup);
        let _ = std::fs::rename(&self.path, &backup);
        self.file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .ok();
        self.written = 0;
    }

    pub fn flush(&mut self) {
        if let Some(f) = self.file.as_mut() {
            let _ = f.flush();
        }
    }
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

/// Return up to `lines` most recent lines from a log file, oldest first.
pub fn tail(path: &Path, lines: usize) -> Vec<String> {
    match std::fs::read(path) {
        Ok(bytes) => {
            let all: Vec<String> = String::from_utf8_lossy(&bytes)
                .lines()
                .map(|l| l.to_string())
                .collect();
            let start = all.len().saturating_sub(lines);
            all[start..].to_vec()
        }
        Err(_) => Vec::new(),
    }
}

/// List available processes that have logs for a workspace.
pub fn logged_processes(dir: &Path) -> Vec<String> {
    let mut out = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.path().file_stem().map(|s| s.to_string_lossy().to_string()))
                .filter(|stem| stem.ends_with(".out") || stem.ends_with(".err"))
                .map(|stem| stem.trim_end_matches(".out").trim_end_matches(".err").to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_bounds_file_size() {
        let dir = std::env::temp_dir().join(format!("lotus-logs-{}-{}", std::process::id(), crate::util::now_ms()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("api.out.log");
        let mut writer = LogWriter::open(path.clone()).unwrap();
        writer.max_bytes = 200;
        let big = "x".repeat(80);
        for _ in 0..10 {
            writer.write_line(big.as_bytes());
            writer.write_line(b"\n");
        }
        writer.flush();
        let current = std::fs::metadata(&path).unwrap().len();
        assert!(current <= 200 + 81, "current size {current}");
        assert!(sibling(&path, ".1").exists(), "rotation happened");
    }
}
