//! Deterministic process supervisor.
//!
//! One supervised workspace = one `run_supervised` execution (a detached
//! `lotus __supervise <key>` child process when started from the CLI, or a
//! background thread when started from the desktop app). The supervisor owns:
//!
//! - dependency-ordered startup (topological order from the manifest)
//! - bounded health polling and workspace state derivation
//! - restart policy with bounded retries and linear backoff
//! - graceful stop first, forced tree termination after the grace period
//! - crash detection with exit-code classification
//! - a heartbeat status file that readers use to detect a dead supervisor
//!
//! There are no OS signals in the supervision protocol: stop requests arrive
//! through `control.json`, which keeps behavior identical across platforms
//! and across CLI/app invocation modes.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::Duration;

use serde::Serialize;

use crate::error::{LotusError, Result};
use crate::ledger::{self, Event};
use crate::logs::LogWriter;
use crate::manifest::{Manifest, Process, RestartPolicy};
use crate::paths::{self, Paths};
use crate::platform;
use crate::registry;
use crate::status::{
    read_control, ControlAction, ProcessState, ProcessStatus, StatusReport, WorkspaceState,
};
use crate::trust;
use crate::util::{format_ts, now_ms};

const TICK_MS: u64 = 250;

#[derive(Debug, Serialize)]
struct ProcEntry {
    name: String,
    state: ProcessState,
    pid: Option<u32>,
    identity_token: u64,
    healthy: Option<bool>,
    restarts_used: u32,
    exit_code: Option<i32>,
    detail: Option<String>,
}

struct ProcRuntime {
    spec: Process,
    entry: ProcEntry,
    child: Option<Child>,
    spawned_at_ms: u64,
    next_health_at_ms: u64,
    pending_restart_at_ms: Option<u64>,
}

impl ProcRuntime {
    fn new(spec: Process) -> Self {
        ProcRuntime {
            entry: ProcEntry {
                name: spec.name.clone(),
                state: ProcessState::Pending,
                pid: None,
                identity_token: 0,
                healthy: None,
                restarts_used: 0,
                exit_code: None,
                detail: None,
            },
            spec,
            child: None,
            spawned_at_ms: 0,
            next_health_at_ms: 0,
            pending_restart_at_ms: None,
        }
    }

    fn alive(&self) -> bool {
        matches!(
            self.entry.state,
            ProcessState::Starting
                | ProcessState::Running
                | ProcessState::Unhealthy
                | ProcessState::Restarting
                | ProcessState::Stopping
        )
    }
}

/// Supervise workspace `key` until stopped. Blocking.
///
/// Any startup failure (bad manifest, lost trust, lock contention, dead
/// platform tooling) leaves a visible terminal `failed` status behind so
/// readers never see an eternal `<no status>` void.
pub fn run_supervised(base: &Path, key: &str) -> Result<()> {
    let layout = paths::layout(base);
    match supervise(&layout, key) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = write_fatal_status(&layout, key, &e.to_string());
            Err(e)
        }
    }
}

fn supervise(layout: &Paths, key: &str) -> Result<()> {
    let ws = registry::get(&layout.registry, key)
        .ok_or_else(|| LotusError::NotFound(format!("workspace {key} is not registered")))?;
    let root = PathBuf::from(&ws.root);

    // Reload the manifest fresh; trust must match what is on disk right now.
    let (manifest, hash) = Manifest::load(&root.join("lotus.toml"))?;
    let entry = trust::get_entry(&layout.trust, key).ok_or_else(|| {
        LotusError::Trust(format!("workspace `{}` is not trusted", manifest.name))
    })?;
    if entry.manifest_hash != hash {
        return Err(LotusError::Trust(format!(
            "manifest changed since it was last trusted ({} -> {}); run `lotus trust` to review and re-approve",
            entry.manifest_hash.get(..8).unwrap_or("?"),
            hash.get(..8).unwrap_or("?"),
        )));
    }

    let _guard = LockGuard::acquire(layout, key)?;

    ledger::append(
        &layout.ledger,
        key,
        Event::new("start_requested").with_detail(serde_json::json!({ "hash": hash })),
    )?;

    let mut procs: Vec<ProcRuntime> = manifest
        .start_order
        .iter()
        .filter_map(|name| manifest.find_process(name))
        .map(|spec| ProcRuntime::new(spec.clone()))
        .collect();

    // Port diagnostics are advisory: a discovery outage must never prevent
    // startup. Conflicts still surface via doctor when tooling recovers.
    let port_conflicts = match preflight_ports(layout, key, &manifest) {
        Ok(c) => c,
        Err(e) => {
            ledger::append(
                &layout.ledger,
                key,
                Event::new("port_discovery_unavailable")
                    .with_detail(serde_json::json!({ "detail": e.to_string() })),
            )
            .ok();
            Vec::new()
        }
    };

    let mut stopping = false;
    let mut spawned_count = 0usize;
    let started_at = now_ms();
    let mut last_state = WorkspaceState::Off;

    loop {
        if !stopping && stop_requested(layout, key) {
            stopping = true;
            ledger::append(&layout.ledger, key, Event::new("stop_requested"))?;
            // Immediate heartbeat so observers see STOPPING before the
            // shutdown sequence blocks on per-process grace periods.
            let st = derive_workspace_state(&procs, true, spawned_count == procs.len());
            write_status(
                layout,
                key,
                &ws.name,
                &root,
                &hash,
                &procs,
                st,
                &port_conflicts,
                started_at,
            )?;
        }

        if !stopping {
            spawned_count += spawn_ready(&mut procs, spawned_count, &root, layout, key);
        }

        poll_exits(&mut procs, layout, key, stopping);
        run_health_checks(&mut procs, &root);
        schedule_restarts(&mut procs, &root, layout, key);

        if stopping {
            shutdown_all(&mut procs, layout, key);
        }

        let state = derive_workspace_state(&procs, stopping, spawned_count == procs.len());
        if state != last_state {
            record_transition(layout, key, last_state, state)?;
            last_state = state;
        }

        write_status(
            layout,
            key,
            &ws.name,
            &root,
            &hash,
            &procs,
            state,
            &port_conflicts,
            started_at,
        )?;

        if stopping && procs.iter().all(|p| !p.alive()) {
            break;
        }
        std::thread::sleep(Duration::from_millis(TICK_MS));
    }

    write_final_off(layout, key, &ws.name, &root, &hash)
}

// ---------------------------------------------------------------- startup

fn stop_requested(layout: &Paths, key: &str) -> bool {
    matches!(
        read_control(&layout.control_file(key)),
        Some(req) if req.action == ControlAction::Stop
    )
}

/// Spawn processes whose dependencies are all running. Returns how many new
/// spawns (successful or failed) were consumed from the dependency order.
fn spawn_ready(
    procs: &mut [ProcRuntime],
    cursor: usize,
    root: &Path,
    layout: &Paths,
    key: &str,
) -> usize {
    let mut used = 0usize;
    while cursor + used < procs.len() {
        let deps_ok = procs[..cursor + used]
            .iter()
            .all(|p| p.entry.state == ProcessState::Running);
        if !deps_ok {
            break;
        }
        let p = &mut procs[cursor + used];
        spawn_process(p, root, layout, key);
        used += 1;
        if p.entry.state != ProcessState::Running {
            // A failed spawn cannot satisfy downstream dependencies.
            break;
        }
    }
    used
}

fn spawn_process(p: &mut ProcRuntime, _root: &Path, layout: &Paths, key: &str) {
    p.entry.detail = None;

    if let Some(env_file) = &p.spec.env_file {
        if !env_file.exists() {
            p.entry.state = ProcessState::Failed;
            p.entry.detail = Some(format!(
                "declared env file {} does not exist",
                env_file.display()
            ));
            return;
        }
    }
    if !p.spec.workdir.is_dir() {
        p.entry.state = ProcessState::Failed;
        p.entry.detail = Some(format!(
            "working directory {} does not exist",
            p.spec.workdir.display()
        ));
        return;
    }
    let exe = match platform::resolve_executable(&p.spec.command) {
        Ok(e) => e,
        Err(e) => {
            p.entry.state = ProcessState::Failed;
            p.entry.detail = Some(e.to_string());
            return;
        }
    };

    let mut command = build_platform_command(&exe, &p.spec.args);
    command.current_dir(&p.spec.workdir);
    // Parent environment first, then env file, then inline env (later wins).
    for (k, v) in std::env::vars_os() {
        command.env(k, v);
    }
    if let Some(env_file) = &p.spec.env_file {
        for (k, v) in parse_env_file(env_file) {
            command.env(k, v);
        }
    }
    for (k, v) in &p.spec.env {
        command.env(k, v);
    }
    command.stdin(std::process::Stdio::null());
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    configure_detach(&mut command);

    match command.spawn() {
        Ok(mut child) => {
            p.entry.pid = Some(child.id());
            p.entry.identity_token = platform::pid_identity_token(child.id()).unwrap_or(0);
            p.spawned_at_ms = now_ms();
            p.next_health_at_ms = now_ms();
            p.pending_restart_at_ms = None;
            p.entry.exit_code = None;
            p.entry.healthy = None;
            p.entry.state = ProcessState::Running;

            let log_dir = layout.logs.join(key);
            if let Some(out) = child.stdout.take() {
                spawn_log_reader(out, "out", &log_dir, &p.spec.name);
            }
            if let Some(err) = child.stderr.take() {
                spawn_log_reader(err, "err", &log_dir, &p.spec.name);
            }
            p.child = Some(child);

            let _ = ledger::append(
                &layout.ledger,
                key,
                Event::for_process("process_spawned", &p.spec.name)
                    .with_detail(serde_json::json!({ "pid": p.entry.pid })),
            );
        }
        Err(e) => {
            p.entry.state = ProcessState::Failed;
            p.entry.detail = Some(format!("spawn failed: {e}"));
            let _ = ledger::append(
                &layout.ledger,
                key,
                Event::for_process("spawn_failed", &p.spec.name)
                    .with_detail(serde_json::json!({ "detail": e.to_string() })),
            );
        }
    }
}

fn parse_env_file(path: &Path) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                pairs.push((k.trim().to_string(), v.trim().to_string()));
            }
        }
    }
    pairs
}

fn build_platform_command(exe: &Path, args: &[String]) -> std::process::Command {
    #[cfg(target_os = "windows")]
    {
        let batch = exe
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("bat") || e.eq_ignore_ascii_case("cmd"))
            .unwrap_or(false);
        if batch {
            let mut cmd = std::process::Command::new("cmd");
            cmd.arg("/C").arg(exe);
            cmd.args(args);
            return cmd;
        }
    }
    let mut cmd = std::process::Command::new(exe);
    cmd.args(args);
    cmd
}

fn configure_detach(command: &mut std::process::Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // New session/process group so pgid == pid and tree signals reach
        // grandchildren.
        let _ = command.process_group(0);
    }
}

fn spawn_log_reader<R: std::io::Read + Send + 'static>(
    reader: R,
    stream_tag: &str,
    log_dir: &Path,
    process_name: &str,
) {
    let tag = stream_tag.to_string();
    let name = process_name.to_string();
    let dir = log_dir.to_path_buf();
    std::thread::spawn(move || {
        let Ok(mut writer) = LogWriter::open(dir.join(format!("{name}.{tag}.log"))) else {
            return;
        };
        for line in BufReader::new(reader).lines() {
            match line {
                Ok(l) => {
                    let stamped = format!("[{}] {}\n", format_ts(now_ms()), l);
                    writer.write_line(stamped.as_bytes());
                }
                Err(_) => break,
            }
        }
        writer.flush();
    });
}

// ---------------------------------------------------------------- monitoring

fn poll_exits(procs: &mut [ProcRuntime], layout: &Paths, key: &str, stopping: bool) {
    for p in procs.iter_mut() {
        let Some(child) = p.child.as_mut() else { continue };
        let Ok(status) = child.try_wait() else { continue };
        let Some(exit) = status else { continue };
        let code = exit.code();
        p.child = None;
        p.entry.pid = None;
        p.entry.exit_code = code;
        p.entry.state = if stopping {
            ProcessState::Stopped
        } else if code == Some(0) {
            ProcessState::Exited
        } else {
            ProcessState::Crashed
        };
        let kind = match p.entry.state {
            ProcessState::Stopped => "process_stopped",
            ProcessState::Exited => "process_exited",
            _ => "crash_detected",
        };
        let _ = ledger::append(
            &layout.ledger,
            key,
            Event::for_process(kind, &p.spec.name)
                .with_detail(serde_json::json!({ "exit_code": code })),
        );
    }
}

fn run_health_checks(procs: &mut [ProcRuntime], root: &Path) {
    let now = now_ms();
    for p in procs.iter_mut() {
        let Some(spec) = p.spec.health.clone() else {
            // No health declared: running counts as healthy.
            if p.entry.state == ProcessState::Running {
                p.entry.healthy = Some(true);
            }
            continue;
        };
        if p.entry.state != ProcessState::Running || now < p.next_health_at_ms {
            continue;
        }
        p.next_health_at_ms = now + spec.interval_ms;
        let results = crate::health::evaluate_all(&spec, root);
        let passed = results.iter().all(|(_, o)| o.passed);
        let within_grace = now.saturating_sub(p.spawned_at_ms) < spec.startup_grace_ms;
        if passed {
            if p.entry.healthy != Some(true) {
                p.entry.healthy = Some(true);
                if p.entry.state == ProcessState::Unhealthy {
                    p.entry.state = ProcessState::Running;
                }
            }
        } else if !within_grace && p.entry.healthy != Some(false) {
            p.entry.healthy = Some(false);
            p.entry.state = ProcessState::Unhealthy;
            p.entry.detail = results
                .iter()
                .find(|(_, o)| !o.passed)
                .map(|(id, o)| format!("{id}: {}", o.detail));
        }
    }
}

fn schedule_restarts(procs: &mut [ProcRuntime], root: &Path, layout: &Paths, key: &str) {
    let now = now_ms();
    for p in procs.iter_mut() {
        if p.entry.state == ProcessState::Crashed
            && p.spec.restart.policy == RestartPolicy::OnFailure
        {
            if p.entry.restarts_used < p.spec.restart.max_restarts {
                p.entry.restarts_used += 1;
                p.entry.state = ProcessState::Restarting;
                p.pending_restart_at_ms =
                    Some(now + p.spec.restart.backoff_ms * p.entry.restarts_used as u64);
                let _ = ledger::append(
                    &layout.ledger,
                    key,
                    Event::for_process("restart_scheduled", &p.spec.name).with_detail(
                        serde_json::json!({
                            "attempt": p.entry.restarts_used,
                            "max": p.spec.restart.max_restarts
                        }),
                    ),
                );
            } else {
                p.entry.state = ProcessState::Failed;
                p.entry.detail = Some(format!(
                    "crashed; restart budget exhausted ({}/{})",
                    p.entry.restarts_used, p.spec.restart.max_restarts
                ));
                let _ = ledger::append(
                    &layout.ledger,
                    key,
                    Event::for_process("restart_exhausted", &p.spec.name),
                );
            }
        }

        if p.entry.state == ProcessState::Restarting
            && p.pending_restart_at_ms.is_some_and(|due| now >= due)
        {
            spawn_process(p, root, layout, key);
        }
    }
}

// ---------------------------------------------------------------- shutdown

fn shutdown_all(procs: &mut [ProcRuntime], layout: &Paths, key: &str) {
    // Reverse dependency order.
    for p in procs.iter_mut().rev() {
        if p.entry.state == ProcessState::Pending {
            p.entry.state = ProcessState::Stopped;
            continue;
        }
        if !p.alive() {
            continue;
        }
        p.entry.state = ProcessState::Stopping;
        if let Some(pid) = p.entry.pid {
            let grace = Duration::from_secs(p.spec.shutdown.grace_secs);
            platform::terminate_tree(pid, grace);
            let _ = ledger::append(
                &layout.ledger,
                key,
                Event::for_process("process_stopped", &p.spec.name),
            );
        }
        p.entry.pid = None;
        p.entry.state = ProcessState::Stopped;
    }
}

// ---------------------------------------------------------------- state

fn derive_workspace_state(
    procs: &[ProcRuntime],
    stopping: bool,
    all_spawned: bool,
) -> WorkspaceState {
    if stopping {
        return WorkspaceState::Stopping;
    }
    if procs.iter().any(|p| p.entry.state.is_terminal_bad()) {
        return WorkspaceState::Failed;
    }
    if !all_spawned
        || procs
            .iter()
            .any(|p| matches!(p.entry.state, ProcessState::Pending | ProcessState::Restarting))
    {
        return WorkspaceState::Starting;
    }
    if procs.iter().any(|p| {
        matches!(
            p.entry.state,
            ProcessState::Unhealthy | ProcessState::Crashed | ProcessState::Exited
        )
    }) {
        return WorkspaceState::Degraded;
    }
    if !procs.is_empty()
        && procs
            .iter()
            .all(|p| p.entry.state == ProcessState::Running && p.entry.healthy == Some(true))
    {
        WorkspaceState::Healthy
    } else {
        WorkspaceState::Starting
    }
}

// ---------------------------------------------------------------- persistence

#[allow(clippy::too_many_arguments)]
fn write_status(
    layout: &Paths,
    key: &str,
    name: &str,
    root: &Path,
    hash: &str,
    procs: &[ProcRuntime],
    state: WorkspaceState,
    conflicts: &[crate::ports::PortConflict],
    started_at: u64,
) -> Result<()> {
    let report = StatusReport {
        key: key.to_string(),
        name: name.to_string(),
        root: root.to_string_lossy().to_string(),
        manifest_hash: hash.to_string(),
        state: state_name(state),
        started_at_ms: Some(started_at),
        updated_at_ms: now_ms(),
        processes: procs
            .iter()
            .map(|p| ProcessStatus {
                name: p.spec.name.clone(),
                state: process_state_name(p.entry.state),
                pid: p.entry.pid,
                identity_token: p.entry.identity_token,
                healthy: p.entry.healthy,
                restarts: p.entry.restarts_used,
                exit_code: p.entry.exit_code,
                detail: p.entry.detail.clone(),
            })
            .collect(),
        port_conflicts: conflicts.to_vec(),
        last_error: procs
            .iter()
            .find(|p| p.entry.state.is_terminal_bad())
            .and_then(|p| p.entry.detail.clone()),
    };
    write_report(layout.status_file(key), &report)
}

fn write_final_off(layout: &Paths, key: &str, name: &str, root: &Path, hash: &str) -> Result<()> {
    let report = StatusReport {
        key: key.to_string(),
        name: name.to_string(),
        root: root.to_string_lossy().to_string(),
        manifest_hash: hash.to_string(),
        state: "off".into(),
        started_at_ms: None,
        updated_at_ms: now_ms(),
        processes: vec![],
        port_conflicts: vec![],
        last_error: None,
    };
    write_report(layout.status_file(key), &report)?;
    let _ = std::fs::remove_file(layout.control_file(key));
    Ok(())
}

fn write_report(path: PathBuf, report: &StatusReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_vec_pretty(report)?)?;
    Ok(())
}

/// Terminal status after a fatal supervisor startup error. Skipped when a
/// live heartbeat already exists (never clobber a running supervisor).
fn write_fatal_status(layout: &Paths, key: &str, error: &str) -> Result<()> {
    if crate::status::read(&layout.status_file(key))
        .map(|s| s.fresh())
        .unwrap_or(false)
    {
        return Ok(());
    }
    let (name, root) = match registry::get(&layout.registry, key) {
        Some(ws) => (ws.name, ws.root),
        None => (key.to_string(), String::new()),
    };
    let report = StatusReport {
        key: key.to_string(),
        name,
        root,
        manifest_hash: String::new(),
        state: "failed".into(),
        started_at_ms: None,
        updated_at_ms: now_ms(),
        processes: vec![],
        port_conflicts: vec![],
        last_error: Some(error.to_string()),
    };
    write_report(layout.status_file(key), &report)
}

fn state_name(state: WorkspaceState) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn process_state_name(state: ProcessState) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn preflight_ports(layout: &Paths, key: &str, manifest: &Manifest) -> Result<Vec<crate::ports::PortConflict>> {
    let expected: Vec<(u16, String)> = manifest
        .processes
        .iter()
        .flat_map(|p| p.ports.iter().map(|port| (*port, p.name.clone())))
        .collect();
    if expected.is_empty() {
        return Ok(vec![]);
    }
    let conflicts = crate::ports::detect_conflicts(&expected, &[])?;
    for c in &conflicts {
        ledger::append(
            &layout.ledger,
            key,
            Event::new("port_conflict_detected")
                .with_process(&c.expected_owner)
                .with_detail(serde_json::json!({
                    "port": c.port,
                    "owner_pid": c.owner_pid,
                    "owner_name": c.owner_name
                })),
        )?;
    }
    Ok(conflicts)
}

fn record_transition(layout: &Paths, key: &str, from: WorkspaceState, to: WorkspaceState) -> Result<()> {
    ledger::append(
        &layout.ledger,
        key,
        Event::new("workspace_state_changed").with_detail(serde_json::json!({
            "from": format!("{from:?}").to_lowercase(),
            "to": format!("{to:?}").to_lowercase()
        })),
    )?;
    Ok(())
}

// ---------------------------------------------------------------- locking

struct LockGuard {
    path: PathBuf,
}

impl LockGuard {
    fn acquire(layout: &Paths, key: &str) -> Result<Self> {
        let lock_path = layout.lock_file(key);
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(mut f) => {
                use std::io::Write;
                let _ = writeln!(f, "pid={}", std::process::id());
                Ok(LockGuard { path: lock_path })
            }
            Err(_) => {
                // Existing lock: fresh heartbeat means genuinely running;
                // stale means a dead supervisor left orphans behind.
                let fresh = crate::status::read(&layout.status_file(key))
                    .map(|s| s.fresh())
                    .unwrap_or(false);
                if fresh {
                    return Err(LotusError::Conflict(format!(
                        "workspace `{key}` is already being supervised"
                    )));
                }
                cleanup_orphans(layout, key);
                std::fs::remove_file(&lock_path)
                    .map_err(|e| LotusError::State(format!("cannot clear stale lock: {e}")))?;
                std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&lock_path)
                    .map(|_| LockGuard { path: lock_path })
                    .map_err(|e| {
                        LotusError::Conflict(format!("cannot acquire supervisor lock: {e}"))
                    })
            }
        }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Kill processes recorded by the last heartbeat after verifying each PID's
/// identity token. This is how LotusOS recovers when ITSELF crashed while the
/// workspace kept running. PIDs whose identity cannot be verified are never
/// touched.
pub fn cleanup_orphans(layout: &Paths, key: &str) -> Vec<String> {
    let mut report = Vec::new();
    let Some(last) = crate::status::read(&layout.status_file(key)) else {
        return report;
    };
    for (pid, token) in last.live_pids() {
        if token == 0 {
            report.push(format!(
                "skipped pid {pid}: no identity token recorded; refusing to kill an unverifiable process"
            ));
            continue;
        }
        if platform::pid_matches_token(pid, token) {
            platform::terminate_tree_verified(pid, token);
            report.push(format!("terminated orphaned pid {pid} (identity verified)"));
        } else {
            report.push(format!("skipped pid {pid}: identity no longer matches"));
        }
    }
    report
}
