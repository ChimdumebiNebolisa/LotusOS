//! The single engine facade used by BOTH the CLI and the desktop app.
//! There is exactly one implementation of workspace operations here.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::checkpoint;
use crate::doctor::{Finding, FindingStatus};
use crate::error::{LotusError, Result};
use crate::ledger;
use crate::manifest::Manifest;
use crate::paths::{self, Paths};
use crate::registry::{self, RegisteredWorkspace};
use crate::status::{self, StatusReport};
use crate::trust::{self, TrustedEntry};
use crate::util::format_ts;
use crate::supervisor;

pub struct Engine {
    pub base: PathBuf,
    pub layout: Paths,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListEntry {
    pub key: String,
    pub name: String,
    pub root: String,
    pub added_at: String,
    pub state: String,
    pub supervisor_alive: bool,
    pub trusted: bool,
    pub manifest_drift: bool,
}

impl Engine {
    pub fn new() -> Self {
        let base = paths::data_dir();
        let layout = paths::layout(&base);
        Engine { base, layout }
    }

    pub fn with_base(base: &Path) -> Self {
        Engine {
            base: base.to_path_buf(),
            layout: paths::layout(base),
        }
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        self.layout.ensure()?;
        Ok(())
    }

    // ------------------------------------------------------------ registration

    /// Load + validate the manifest at `root`, optionally granting trust,
    /// and register the workspace.
    pub fn add(&self, root: &Path, grant_trust: bool) -> Result<String> {
        self.ensure_dirs()?;
        let manifest_path = root.join("lotus.toml");
        if !manifest_path.is_file() {
            return Err(LotusError::NotFound(format!(
                "no lotus.toml found at {}",
                root.display()
            )));
        }
        let (manifest, hash) = Manifest::load(&manifest_path)?;
        let key = trust::workspace_key(root);

        registry::add(&self.layout.registry, &key, &manifest.name, root)?;
        if grant_trust {
            trust::grant(&self.layout.trust, &key, root, &manifest.name, &hash)?;
        }
        ledger::append(
            &self.layout.ledger,
            &key,
            ledger::Event::new(if grant_trust { "workspace_added" } else { "workspace_added_untrusted" })
                .with_detail(serde_json::json!({ "name": manifest.name, "hash": hash })),
        )?;
        Ok(key)
    }

    pub fn remove(&self, selector: &str) -> Result<()> {
        let (key, _) = self.resolve(selector)?;
        registry::remove(&self.layout.registry, &key)?;
        trust::revoke(&self.layout.trust, &key)?;
        Ok(())
    }

    pub fn resolve(&self, selector: &str) -> Result<(String, RegisteredWorkspace)> {
        registry::find_by_name_or_key(&self.layout.registry, selector)
            .ok_or_else(|| LotusError::NotFound(format!("no registered workspace matches `{selector}`")))
    }

    pub fn list(&self) -> Vec<ListEntry> {
        let mut entries: Vec<ListEntry> = registry::all(&self.layout.registry)
            .into_iter()
            .map(|(key, ws)| {
                let status = status::read(&self.layout.status_file(&key));
                let state = status
                    .as_ref()
                    .map(|s| s.state.clone())
                    .unwrap_or_else(|| "off".into());
                let alive = status.as_ref().map(|s| s.fresh()).unwrap_or(false);
                let manifest_hash = Manifest::load(&PathBuf::from(&ws.root).join("lotus.toml"))
                    .ok()
                    .map(|(m, _)| m.hash);
                let trusted_entry = trust::get_entry(&self.layout.trust, &key);
                let manifest_drift = matches!(
                    (&trusted_entry, &manifest_hash),
                    (Some(t), Some(h)) if t.manifest_hash != *h
                );
                ListEntry {
                    key,
                    name: ws.name.clone(),
                    root: ws.root.clone(),
                    added_at: format_ts(ws.added_at_ms),
                    state,
                    supervisor_alive: alive,
                    trusted: trusted_entry.is_some(),
                    manifest_drift,
                }
            })
            .collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries
    }

    // ------------------------------------------------------------ trust

    pub fn trust_entry(&self, key: &str) -> Option<TrustedEntry> {
        trust::get_entry(&self.layout.trust, key)
    }

    /// Load the current manifest for display before a trust decision.
    pub fn manifest_for_review(&self, selector: &str) -> Result<(String, Manifest)> {
        let (key, ws) = self.resolve(selector)?;
        let (manifest, _) = Manifest::load(&PathBuf::from(ws.root).join("lotus.toml"))?;
        Ok((key, manifest))
    }

    pub fn grant_trust(&self, selector: &str) -> Result<()> {
        let (key, ws) = self.resolve(selector)?;
        let (manifest, hash) = Manifest::load(&PathBuf::from(&ws.root).join("lotus.toml"))?;
        trust::grant(&self.layout.trust, &key, Path::new(&ws.root), &manifest.name, &hash)?;
        ledger::append(&self.layout.ledger, &key, ledger::Event::new("trust_granted"))?;
        Ok(())
    }

    // ------------------------------------------------------------ lifecycle

    /// Start via a detached `__supervise` child of the current executable.
    pub fn start_detached(&self, selector: &str) -> Result<()> {
        let (key, _) = self.resolve(selector)?;
        self.pre_start_checks(&key)?;

        #[cfg(target_os = "windows")]
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        let mut command = std::process::Command::new(
            std::env::current_exe()
                .map_err(|e| LotusError::State(format!("cannot locate own executable: {e}")))?,
        );
        command.args(["__supervise", &key]);
        command.stdin(std::process::Stdio::null());
        command.stdout(std::process::Stdio::null());
        command.stderr(std::process::Stdio::null());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(DETACHED_PROCESS | 0x0000_0200); // detached + new group
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let _ = command.process_group(0);
        }
        command
            .spawn()
            .map_err(|e| LotusError::State(format!("failed to launch supervisor: {e}")))?;
        Ok(())
    }

    /// Start supervision on a background thread of THIS process (desktop app mode).
    pub fn start_in_thread(&self, selector: &str) -> Result<std::thread::JoinHandle<()>> {
        let (key, _) = self.resolve(selector)?;
        self.pre_start_checks(&key)?;
        let base = self.base.clone();
        Ok(std::thread::spawn(move || {
            if let Err(e) = supervisor::run_supervised(&base, &key) {
                eprintln!("lotus supervisor error: {e}");
            }
        }))
    }

    fn pre_start_checks(&self, key: &str) -> Result<()> {
        self.ensure_dirs()?;
        let (_, ws) = self.resolve(key)?;
        let (manifest, hash) = Manifest::load(&PathBuf::from(ws.root).join("lotus.toml"))?;

        let entry = self
            .trust_entry(key)
            .ok_or_else(|| LotusError::Trust(format!(
                "workspace `{}` has never been trusted; run `lotus trust <workspace>` after reviewing its commands",
                manifest.name
            )))?;
        if entry.manifest_hash != hash {
            return Err(LotusError::Trust(format!(
                "manifest changed since last trust decision ({} -> {}); re-run `lotus trust`",
                entry.manifest_hash.get(..8).unwrap_or("?"),
                hash.get(..8).unwrap_or("?"),
            )));
        }

        // Refuse double-start while a live heartbeat shows an active run.
        // Terminal states (off/stopping/failed) are always restartable.
        if let Some(s) = status::read(&self.layout.status_file(key)) {
            if s.fresh() && !matches!(s.state.as_str(), "off" | "stopping" | "failed") {
                return Err(LotusError::Conflict(format!(
                    "workspace `{}` is already running",
                    s.name
                )));
            }
        }
        Ok(())
    }

    pub fn stop(&self, selector: &str) -> Result<Vec<String>> {
        let (key, ws) = self.resolve(selector)?;
        let mut notes = Vec::new();

        let live_supervisor = status::read(&self.layout.status_file(&key))
            .map(|s| s.fresh())
            .unwrap_or(false);

        if live_supervisor {
            status::request_stop(&self.layout.control_file(&key))?;
            // Wait for OFF (bounded by manifest shutdown budgets + slack).
            let budget_ms = Manifest::load(&PathBuf::from(ws.root).join("lotus.toml"))
                .map(|(m, _)| {
                    m.processes.iter().map(|p| p.shutdown.grace_secs * 1000).sum::<u64>()
                })
                .unwrap_or(10_000)
                .max(2_000)
                + 6_000;
            let deadline = Instant::now() + Duration::from_millis(budget_ms);
            loop {
                match status::read(&self.layout.status_file(&key)) {
                    Some(s) if s.state == "off" => break,
                    Some(s) if !s.fresh() => {
                        notes.push("supervisor died during stop; falling back to orphan cleanup".into());
                        break;
                    }
                    None => break,
                    _ => {}
                }
                if Instant::now() >= deadline {
                    notes.push(format!(
                        "stop request timed out after {}ms; workspace may still be stopping",
                        budget_ms
                    ));
                    break;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        } else {
            notes.push("no live supervisor detected".into());
        }

        // If processes survived (crashed supervisor), clean them up safely.
        notes.extend(supervisor::cleanup_orphans(&self.layout, &key));
        ledger::append(&self.layout.ledger, &key, ledger::Event::new("stop_completed"))?;
        Ok(notes)
    }

    pub fn restart(&self, selector: &str) -> Result<()> {
        let currently_running = self
            .status(selector)
            .map(|s| s.fresh() && s.state != "off")
            .unwrap_or(false);
        if currently_running {
            self.stop(selector)?;
        }
        self.start_detached(selector)
    }

    // ------------------------------------------------------------ inspection

    pub fn status(&self, selector: &str) -> Result<StatusReport> {
        let (key, _) = self.resolve(selector)?;
        status::read(&self.layout.status_file(&key)).ok_or_else(|| {
            LotusError::State(format!("workspace `{selector}` has never been started"))
        })
    }

    pub fn doctor(&self, selector: &str) -> Result<Vec<Finding>> {
        let (key, ws) = self.resolve(selector)?;
        let (manifest, _) = Manifest::load(&PathBuf::from(ws.root).join("lotus.toml"))?;
        let mut findings = crate::doctor::run(&manifest)?;

        // Trust state as a doctor-visible finding too.
        findings.push(match self.trust_entry(&key) {
            Some(entry) if entry.manifest_hash == manifest.hash => Finding {
                check: "trust".into(),
                subject: manifest.name.clone(),
                status: FindingStatus::Ok,
                message: format!("trusted manifest ({})", short(&manifest.hash)),
            },
            Some(entry) => Finding {
                check: "trust".into(),
                subject: manifest.name.clone(),
                status: FindingStatus::Invalid,
                message: format!(
                    "manifest changed since trust ({} -> {})",
                    short(&entry.manifest_hash),
                    short(&manifest.hash)
                ),
            },
            None => Finding {
                check: "trust".into(),
                subject: manifest.name.clone(),
                status: FindingStatus::Missing,
                message: "never trusted; start will be refused".into(),
            },
        });
        Ok(findings)
    }

    pub fn events(&self, selector: &str, limit: usize) -> Result<Vec<ledger::Event>> {
        let (key, _) = self.resolve(selector)?;
        Ok(ledger::tail(&self.layout.ledger, &key, limit))
    }

    pub fn log_files(&self, key: &str) -> Vec<String> {
        crate::logs::logged_processes(&self.layout.logs.join(key))
    }

    /// Tail logs: returns (lines, source_label). `process` filters to one
    /// process; otherwise the most recently written stream wins.
    pub fn tail_logs(&self, selector: &str, process: Option<&str>, lines: usize) -> Result<Vec<String>> {
        let (key, _) = self.resolve(selector)?;
        let dir = self.layout.logs.join(&key);
        let procs = crate::logs::logged_processes(&dir);
        if procs.is_empty() {
            return Ok(vec!["(no log output yet)".into()]);
        }
        let target = match process {
            Some(name) => {
                if !procs.iter().any(|p| p == name) {
                    return Err(LotusError::NotFound(format!(
                        "no logs for process `{name}`; available: {}",
                        procs.join(", ")
                    )));
                }
                vec![name.to_string()]
            }
            None => vec![procs.last().expect("nonempty").clone()],
        };
        let mut merged: Vec<(u64, String)> = Vec::new();
        for name in target {
            for tag in ["out", "err"] {
                let path = dir.join(format!("{name}.{tag}.log"));
                let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
                let ts = mtime
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                for line in crate::logs::tail(&path, lines) {
                    merged.push((ts, format!("{name}/{tag}: {line}")));
                }
            }
        }
        merged.sort_by_key(|(_, l)| l.clone());
        let start = merged.len().saturating_sub(lines);
        Ok(merged[start..].iter().map(|(_, l)| l.clone()).collect())
    }

    // ------------------------------------------------------------ checkpoints

    pub fn checkpoint_create(&self, selector: &str, note: Option<String>) -> Result<checkpoint::Checkpoint> {
        let (key, ws) = self.resolve(selector)?;
        let (manifest, _) = Manifest::load(&PathBuf::from(ws.root).join("lotus.toml"))?;
        let last_state = status::read(&self.layout.status_file(&key)).map(|s| s.state);
        let cp = checkpoint::create(&self.layout, &key, &manifest, last_state, note)?;
        ledger::append(
            &self.layout.ledger,
            &key,
            ledger::Event::new("checkpoint_created")
                .with_detail(serde_json::json!({ "id": cp.id })),
        )?;
        Ok(cp)
    }

    pub fn checkpoints(&self, selector: &str) -> Result<Vec<checkpoint::Checkpoint>> {
        let (key, _) = self.resolve(selector)?;
        Ok(checkpoint::list(&self.layout, &key))
    }

    pub fn checkpoint_find(&self, selector: &str, cp_selector: &str) -> Result<checkpoint::Checkpoint> {
        let (key, _) = self.resolve(selector)?;
        checkpoint::find(&self.layout, &key, cp_selector)
    }

    pub fn restore_preview(&self, workspace_selector: &str, cp_selector: &str) -> Result<RestorePreview> {
        let (key, _) = self.resolve(workspace_selector)?;
        let cp = checkpoint::find(&self.layout, &key, cp_selector)?;
        let drift = checkpoint::compute_drift(&self.layout, &cp);
        Ok(RestorePreview { checkpoint: cp, drift })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct RestorePreview {
    pub checkpoint: checkpoint::Checkpoint,
    pub drift: Vec<checkpoint::Drift>,
}

fn short(hash: &str) -> String {
    hash.get(..12).unwrap_or(hash).to_string()
}
