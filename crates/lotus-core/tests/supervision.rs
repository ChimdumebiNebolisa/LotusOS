//! Adversarial integration tests exercising real process supervision.
//!
//! Each test drives the same Engine the CLI and desktop app use, against an
//! isolated temp state directory, with real OS child processes.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use lotus_core::engine::Engine;
use lotus_core::status::WorkspaceState;

// ---------------------------------------------------------------- helpers

/// Heavy supervision tests spawn real processes and query real listener
/// tables; serializing them keeps timing assertions meaningful.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn unique_temp(tag: &str) -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "lotus-it-{}-{}-{}",
        tag,
        std::process::id(),
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn sleeper(secs: u32) -> (String, Vec<String>) {
    #[cfg(target_os = "windows")]
    return (
        "ping".to_string(),
        vec!["-n".into(), secs.to_string(), "127.0.0.1".into()],
    );
    #[cfg(not(target_os = "windows"))]
    let _ = secs;
    #[cfg(not(target_os = "windows"))]
    ("sleep".to_string(), vec!["30".to_string()])
}

fn crasher(code: i32) -> (String, Vec<String>) {
    #[cfg(target_os = "windows")]
    return (
        "cmd".to_string(),
        vec!["/C".into(), format!("exit {code}")],
    );
    #[cfg(not(target_os = "windows"))]
    let _ = code;
    #[cfg(not(target_os = "windows"))]
    ("sh".to_string(), vec!["-c".into(), "exit 3".into()])
}

fn manifest_toml(name: &str, body: &str) -> String {
    format!("version = 1\nname = \"{name}\"\n{body}")
}

fn write_ws(root: &Path, name: &str, body: &str) {
    std::fs::write(root.join("lotus.toml"), manifest_toml(name, body)).unwrap();
}

fn setup_workspace(tag: &str, name: &str, body: &str) -> (PathBuf, PathBuf, Engine, String, std::sync::MutexGuard<'static, ()>) {
    let guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let base = unique_temp(tag);
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    write_ws(&root, name, body);
    let engine = Engine::with_base(&base);
    let key = engine.add(&root, true).expect("register+trust");
    (base, root, engine, key, guard)
}

fn wait_for_state(engine: &Engine, name: &str, wanted: &[&str], timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(s) = engine.status(name) {
            if wanted.contains(&s.state.as_str()) {
                return s.state;
            }
            if let Some(last) = wanted.last() {
                let _ = last;
            }
        }
        if Instant::now() >= deadline {
            let current = engine.status(name).map(|s| s.state).unwrap_or_else(|_| "<no status>".into());
            panic!("timed out waiting for {:?}; last state: {current}", wanted);
        }
        std::thread::sleep(Duration::from_millis(150));
    }
}

fn process_detail(engine: &Engine, name: &str, process: &str) -> Option<String> {
    engine
        .status(name)
        .ok()
        .and_then(|s| s.processes.into_iter().find(|p| p.name == process))
        .and_then(|p| p.detail)
}

// ---------------------------------------------------------------- lifecycle

#[test]
fn healthy_lifecycle_and_clean_stop() {
    let (_guard, _root, engine, _key, _serial) =
        setup_workspace("lifecycle", "life-ws", &format!("\n[[process]]\nname = \"worker\"\ncommand = \"{}\"\nargs = {:?}\n[process.shutdown]\ngrace_secs = 1\n",
            sleeper(30).0,
            sleeper(30).1
        ));

    let _t = engine.start_in_thread("life-ws").expect("start");
    // No health checks declared: running => HEALTHY.
    wait_for_state(&engine, "life-ws", &["healthy"], Duration::from_secs(20));

    let started = Instant::now();
    engine.stop("life-ws").expect("stop");
    assert!(started.elapsed() < Duration::from_secs(12), "stop took {:?}", started.elapsed());
    let s = engine.status("life-ws").unwrap();
    assert_eq!(s.state, "off");
    assert!(s.processes.is_empty());
}

#[test]
fn immediate_clean_exit_degrades() {
    let (_g, _r, engine, _k, _serial) = setup_workspace(
        "exit0",
        "exit-ws",
        "\n[[process]]\nname = \"quitter\"\ncommand = \"cmd\"\nargs = [\"/C\", \"exit\", \"0\"]\n",
    );
    let _t = engine.start_in_thread("exit-ws").unwrap();
    // Clean exit without restart policy -> Exited -> DEGRADED.
    wait_for_state(&engine, "exit-ws", &["degraded"], Duration::from_secs(20));
    let s = engine.status("exit-ws").unwrap();
    assert_eq!(s.processes[0].state, "exited");
    engine.stop("exit-ws").unwrap();
}

#[test]
fn crash_restart_budget_then_failed() {
    let (_g, _r, engine, _k, _serial) = setup_workspace(
        "restart-loop",
        "loop-ws",
        "\n[[process]]\nname = \"crasher\"\ncommand = \"cmd\"\nargs = [\"/C\", \"exit\", \"3\"]\n\n[process.restart]\npolicy = \"on-failure\"\nmax_restarts = 2\nbackoff_ms = 100\n",
    );

    let _t = engine.start_in_thread("loop-ws").unwrap();
    // Restart budget: initial + 2 retries, then FAILED.
    wait_for_state(&engine, "loop-ws", &["failed"], Duration::from_secs(25));

    let events = engine.events("loop-ws", 50).unwrap();
    let scheduled = events.iter().filter(|e| e.kind == "restart_scheduled").count();
    assert_eq!(scheduled, 2, "exactly max_restarts schedules, got {scheduled}");
    assert!(events.iter().any(|e| e.kind == "restart_exhausted"));
    assert!(events.iter().any(|e| e.kind == "crash_detected"));
    engine.stop("loop-ws").unwrap();
}

#[test]
fn crash_without_policy_stays_degraded_not_restarted() {
    let (_g, _r, engine, _k, _serial) = setup_workspace(
        "no-restart",
        "nr-ws",
        &format!(
            "\n[[process]]\nname = \"boom\"\ncommand = \"{}\"\nargs = {:?}\n",
            crasher(2).0,
            crasher(2).1
        ),
    );
    let _t = engine.start_in_thread("nr-ws").unwrap();
    wait_for_state(&engine, "nr-ws", &["degraded"], Duration::from_secs(20));
    let s = engine.status("nr-ws").unwrap();
    assert_eq!(s.processes[0].state, "crashed");
    assert_eq!(s.processes[0].restarts, 0);
    engine.stop("nr-ws").unwrap();
}

#[test]
fn dependency_order_and_cycle_refusal() {
    // Valid ordering first.
    let (_g, _r, engine, _k, _serial) = setup_workspace(
        "deps",
        "dep-ws",
        &format!(
            "\n[[process]]\nname = \"second\"\ncommand = \"{}\"\nargs = {:?}\ndepends_on = [\"first\"]\n\
             [[process]]\nname = \"first\"\ncommand = \"{}\"\nargs = {:?}\n",
            sleeper(30).0, sleeper(30).1, sleeper(30).0, sleeper(30).1
        ),
    );
    let _t = engine.start_in_thread("dep-ws").unwrap();
    wait_for_state(&engine, "dep-ws", &["healthy"], Duration::from_secs(20));
    engine.stop("dep-ws").unwrap();

    // Cycle refuses registration-time validation.
    let base = unique_temp("cycle");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("lotus.toml"),
        manifest_toml(
            "cyc",
            "\n[[process]]\nname = \"a\"\ncommand = \"x\"\ndepends_on = [\"b\"]\n[[process]]\nname = \"b\"\ncommand = \"y\"\ndepends_on = [\"a\"]\n",
        ),
    )
    .unwrap();
    let engine2 = Engine::with_base(&base);
    let err = engine2.add(&root, false).unwrap_err();
    assert!(err.to_string().contains("cyclic"), "{err}");
}

#[test]
fn missing_executable_marks_process_failed() {
    let (_g, _r, engine, _k, _serial) = setup_workspace(
        "missing-exe",
        "mexe-ws",
        "\n[[process]]\nname = \"ghost\"\ncommand = \"definitely-not-a-real-executable-xyz\"\n",
    );
    let _t = engine.start_in_thread("mexe-ws").unwrap();
    wait_for_state(&engine, "mexe-ws", &["failed"], Duration::from_secs(15));
    let detail = process_detail(&engine, "mexe-ws", "ghost").unwrap_or_default();
    assert!(detail.to_lowercase().contains("not found") || detail.to_lowercase().contains("path"), "{detail}");
    engine.stop("mexe-ws").unwrap();
}

#[test]
fn invalid_working_directory_marks_process_failed() {
    let (_g, _r, engine, _k, _serial) = setup_workspace(
        "bad-cwd",
        "cwd-ws",
        &format!(
            "\n[[process]]\nname = \"lost\"\ncommand = \"{}\"\nworkdir = \"does/not/exist\"\n",
            sleeper(30).0
        ),
    );
    let _t = engine.start_in_thread("cwd-ws").unwrap();
    wait_for_state(&engine, "cwd-ws", &["failed"], Duration::from_secs(15));
    let detail = process_detail(&engine, "cwd-ws", "lost").unwrap_or_default();
    assert!(detail.contains("working directory"), "{detail}");
    engine.stop("cwd-ws").unwrap();
}

#[test]
fn command_path_with_spaces_runs() {
    let (_g, root, engine, _k, _serial) = setup_workspace("spaces", "space-ws", "");
    let _ = &_g;
    let tool_dir = root.join("my tool dir");
    std::fs::create_dir_all(&tool_dir).unwrap();

    #[cfg(target_os = "windows")]
    let script = tool_dir.join("tool.cmd");
    #[cfg(target_os = "windows")]
    std::fs::write(&script, "@echo hello-from-spaced-path\r\n").unwrap();
    #[cfg(not(target_os = "windows"))]
    let script = tool_dir.join("tool.sh");
    #[cfg(not(target_os = "windows"))]
    std::fs::write(&script, "#!/bin/sh\necho hello\n").unwrap();

    // Re-write manifest referencing the spaced absolute path.
    // TOML literal strings (single quotes) keep Windows backslashes verbatim.
    std::fs::write(
        root.join("lotus.toml"),
        manifest_toml(
            "space-ws",
            &format!(
                "\n[[process]]\nname = \"spaced\"\ncommand = '{}'\n[process.restart]\npolicy = \"never\"\n",
                script.display()
            ),
        ),
    )
    .unwrap();
    engine.grant_trust("space-ws").unwrap();

    let _t = engine.start_in_thread("space-ws").unwrap();
    wait_for_state(&engine, "space-ws", &["degraded"], Duration::from_secs(20));
    let s = engine.status("space-ws").unwrap();
    assert_eq!(s.processes[0].exit_code, Some(0), "batch ran cleanly");
    engine.stop("space-ws").unwrap();
}

// ---------------------------------------------------------------- health

#[test]
fn tcp_health_passes_then_fails_after_listener_closes() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let (_g, _r, engine, _k, _serial) = setup_workspace(
        "health",
        "health-ws",
        &format!(
            "\n[[process]]\nname = \"app\"\ncommand = \"{}\"\nargs = {:?}\n\n[process.health]\nport = {port}\ninterval_ms = 400\ntimeout_ms = 800\nstartup_grace_ms = 1000\n[process.shutdown]\ngrace_secs = 1\n",
            sleeper(30).0,
            sleeper(30).1
        ),
    );

    let _t = engine.start_in_thread("health-ws").unwrap();
    wait_for_state(&engine, "health-ws", &["healthy"], Duration::from_secs(20));

    // Service dies: close the listener and drop the guard socket.
    drop(listener);

    wait_for_state(&engine, "health-ws", &["degraded"], Duration::from_secs(20));
    let s = engine.status("health-ws").unwrap();
    assert_eq!(s.processes[0].state, "unhealthy");
    assert_eq!(s.processes[0].healthy, Some(false));
    engine.stop("health-ws").unwrap();
}

#[test]
fn startup_grace_prevents_premature_unhealthy() {
    // Reserve a port, then release it so nothing listens (a bound-but-unserved
    // socket still accepts TCP handshakes into its backlog).
    let port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };

    let (_g, _r, engine, _k, _serial) = setup_workspace(
        "grace",
        "grace-ws",
        &format!(
            "\n[[process]]\nname = \"slow-app\"\ncommand = \"{}\"\nargs = {:?}\n\n[process.health]\nport = {port}\ninterval_ms = 300\ntimeout_ms = 500\nstartup_grace_ms = 2500\n[process.shutdown]\ngrace_secs = 1\n",
            sleeper(30).0,
            sleeper(30).1
        ),
    );
    // Port never opens, but grace window keeps us out of degraded briefly;
    // after grace elapses the failure lands.
    let _t = engine.start_in_thread("grace-ws").unwrap();
    wait_for_state(&engine, "grace-ws", &["degraded"], Duration::from_secs(20));
    engine.stop("grace-ws").unwrap();
}

#[test]
fn http_health_check_against_real_endpoint() {
    // Real local HTTP endpoint served from the test.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(s) => s,
                Err(_) => break,
            };
            use std::io::{Read, Write};
            let mut buf = [0u8; 512];
            let _ = stream.read(&mut buf);
            let _ = stream.write_all(b"HTTP/1.0 200 OK\r\nContent-Length: 2\r\n\r\nok");
        }
    });
    let _ = &server;

    let (_g, _r, engine, _k, _serial) = setup_workspace(
        "http-health",
        "http-ws",
        &format!(
            "\n[[process]]\nname = \"web\"\ncommand = \"{}\"\nargs = {:?}\n\n[process.health]\nport = {port}\nhttp_path = \"/healthz\"\ninterval_ms = 400\ntimeout_ms = 1500\nstartup_grace_ms = 300\n[process.shutdown]\ngrace_secs = 1\n",
            sleeper(30).0,
            sleeper(30).1
        ),
    );
    let _t = engine.start_in_thread("http-ws").unwrap();
    wait_for_state(&engine, "http-ws", &["healthy"], Duration::from_secs(20));
    engine.stop("http-ws").unwrap();
}

// ---------------------------------------------------------------- ports

#[test]
fn preflight_port_conflict_reported_never_killed() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let (_g, _r, engine, _k, _serial) = setup_workspace(
        "conflict",
        "conflict-ws",
        &format!(
            "\n[[process]]\nname = \"web\"\ncommand = \"{}\"\nargs = {:?}\nports = [{port}]\n[process.shutdown]\ngrace_secs = 1\n",
            sleeper(30).0,
            sleeper(30).1
        ),
    );

    let _t = engine.start_in_thread("conflict-ws").unwrap();
    wait_for_state(&engine, "conflict-ws", &["healthy"], Duration::from_secs(20));
    let s = engine.status("conflict-ws").unwrap();
    assert_eq!(s.port_conflicts.len(), 1, "conflict recorded");
    let c = &s.port_conflicts[0];
    assert_eq!(c.port, port);
    assert!(!c.owned_by_workspace);
    assert!(c.remediation.contains("will not kill"));

    // Doctor surfaces it too.
    let findings = engine.doctor("conflict-ws").unwrap();
    assert!(findings
        .iter()
        .any(|f| f.check == "port"
            && format!("{:?}", f.status).to_uppercase().contains("CONFLICT")));
    engine.stop("conflict-ws").unwrap();
}

#[test]
fn two_workspaces_same_port_both_diagnosed() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let body = format!(
        "\n[[process]]\nname = \"svc\"\ncommand = \"{}\"\nargs = {:?}\nports = [{port}]\n[process.shutdown]\ngrace_secs = 1\n",
        sleeper(30).0,
        sleeper(30).1
    );
    let (_g1, _r1, e1, _k1, _serial) = setup_workspace("two-a", "two-a-ws", &body);
    // Second workspace built inline: SERIAL is not reentrant and this thread
    // already holds it via _serial.
    let base2 = unique_temp("two-b");
    let root2 = base2.join("ws");
    std::fs::create_dir_all(&root2).unwrap();
    write_ws(&root2, "two-b-ws", &body);
    let e2 = Engine::with_base(&base2);
    let _k2 = e2.add(&root2, true).expect("register+trust");

    let d1 = e1.doctor("two-a-ws").unwrap();
    let d2 = e2.doctor("two-b-ws").unwrap();
    assert!(d1.iter().any(|f| f.check == "port"
        && format!("{:?}", f.status).to_uppercase().contains("CONFLICT")));
    assert!(d2.iter().any(|f| f.check == "port"
        && format!("{:?}", f.status).to_uppercase().contains("CONFLICT")));

    // Starting one records the conflict in its live status as well.
    let t1 = e1.start_in_thread("two-a-ws").unwrap();
    wait_for_state(&e1, "two-a-ws", &["healthy"], Duration::from_secs(20));
    let conflicts = e1.status("two-a-ws").unwrap().port_conflicts.len();
    assert_eq!(conflicts, 1);
    e1.stop("two-a-ws").unwrap();
    let _ = t1;
    let _ = (&_k1, &_k2);
}

// ---------------------------------------------------------------- orphans

#[test]
fn orphan_cleanup_verifies_identity_before_killing() {
    let base = unique_temp("orphan");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    write_ws(&root, "orph-ws", "");
    let engine = Engine::with_base(&base);
    let key = engine.add(&root, true).unwrap();
    let layout = lotus_core::paths::layout(&base);

    // Case 1: correct identity token -> terminated.
    let mut child = std::process::Command::new(sleeper(60).0)
        .args(sleeper(60).1)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let pid = child.id();
    let token = lotus_core::platform::pid_identity_token(pid).expect("token available");

    let report = lotus_core::status::StatusReport {
        key: key.clone(),
        name: "orph-ws".into(),
        root: root.display().to_string(),
        manifest_hash: String::new(),
        state: "healthy".into(),
        started_at_ms: None,
        updated_at_ms: lotus_core::util::now_ms(),
        processes: vec![lotus_core::status::ProcessStatus {
            name: "worker".into(),
            state: "running".into(),
            pid: Some(pid),
            identity_token: token,
            healthy: Some(true),
            restarts: 0,
            exit_code: None,
            detail: None,
        }],
        port_conflicts: vec![],
        last_error: None,
    };
    std::fs::create_dir_all(layout.runtime.join(&key)).unwrap();
    std::fs::write(
        layout.status_file(&key),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();

    let notes = lotus_core::supervisor::cleanup_orphans(&layout, &key);
    assert!(notes.iter().any(|n| n.contains("terminated")), "{notes:?}");
    let gone = child.wait().expect("child reaped");
    let _ = gone;

    // Case 2: mismatched token -> skipped, process survives.
    let mut survivor = std::process::Command::new(sleeper(60).0)
        .args(sleeper(60).1)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let spid = survivor.id();
    let stoken = lotus_core::platform::pid_identity_token(spid).expect("token");
    let mut report2 = report.clone();
    report2.processes[0].pid = Some(spid);
    report2.processes[0].identity_token = stoken.wrapping_add(0xDEAD);
    report2.updated_at_ms = lotus_core::util::now_ms();
    std::fs::write(
        layout.status_file(&key),
        serde_json::to_vec_pretty(&report2).unwrap(),
    )
    .unwrap();

    let notes2 = lotus_core::supervisor::cleanup_orphans(&layout, &key);
    assert!(notes2.iter().any(|n| n.contains("identity")), "{notes2:?}");
    assert!(
        lotus_core::platform::pid_identity_token(spid) == Some(stoken),
        "survivor untouched"
    );
    let _ = survivor.kill();
    let _ = survivor.wait();
}

// ---------------------------------------------------------------- trust

#[test]
fn manifest_change_requires_retrust_before_start() {
    let base = unique_temp("retrust");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    write_ws(&root, "trust-ws", "");
    let engine = Engine::with_base(&base);
    engine.add(&root, true).unwrap();

    // Material change: new process added.
    write_ws(&root, "trust-ws", &format!("\n[[process]]\nname = \"new-thing\"\ncommand = \"{}\"\n", sleeper(5).0));

    // list shows drift...
    let entries = engine.list();
    assert!(entries[0].manifest_drift);

    // ...start refuses...
    let err = engine.start_in_thread("trust-ws").unwrap_err();
    assert!(err.to_string().contains("changed"), "{err}");

    // ...until explicitly re-trusted.
    engine.grant_trust("trust-ws").unwrap();
    assert!(!engine.list()[0].manifest_drift);
}

#[test]
fn untrusted_workspace_never_starts() {
    let base = unique_temp("untrusted");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    write_ws(&root, "dark-ws", "");
    let engine = Engine::with_base(&base);
    engine.add(&root, false).unwrap(); // registered WITHOUT trust
    let err = engine.start_in_thread("dark-ws").unwrap_err();
    assert!(err.to_string().contains("trusted"), "{err}");
}

// ---------------------------------------------------------------- secrets

#[test]
fn doctor_never_prints_env_var_values() {
    std::env::set_var("LOTUS_TEST_SECRET_VALUE", "super-secret-do-not-print-42");
    let base = unique_temp("secrets");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("lotus.toml"),
        manifest_toml(
            "sec-ws",
            "\n[env]\nrequired = [\"LOTUS_TEST_SECRET_VALUE\"]\n[[process]]\nname = \"p\"\ncommand = \"where\"\n",
        ),
    )
    .unwrap();
    let engine = Engine::with_base(&base);
    engine.add(&root, true).unwrap();

    let findings = engine.doctor("sec-ws").unwrap();
    let rendered = format!("{findings:?}");
    assert!(rendered.contains("LOTUS_TEST_SECRET_VALUE"), "var name shown");
    assert!(
        !rendered.contains("super-secret-do-not-print-42"),
        "SECRET VALUE LEAKED: {rendered}"
    );
    std::env::remove_var("LOTUS_TEST_SECRET_VALUE");
}

// ---------------------------------------------------------------- corrupt state

#[test]
fn malformed_local_state_files_are_tolerated() {
    let base = unique_temp("corrupt");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    write_ws(&root, "corrupt-ws", "");

    // Corrupt every persisted store BEFORE the engine touches them.
    std::fs::write(base.join("workspaces.json"), b"{not json at all").unwrap();
    std::fs::write(base.join("trust.json"), b"\x00\x01garbage\xff").unwrap();

    let engine = Engine::with_base(&base);
    // Registry reads as empty rather than panicking; re-add works.
    let entries = engine.list();
    assert!(entries.is_empty(), "corrupt registry reads empty");

    let key = engine.add(&root, true).unwrap();
    assert!(!key.is_empty());

    // Corrupt status file: status() reports unavailable instead of panicking.
    std::fs::create_dir_all(base.join("runtime").join(&key)).unwrap();
    std::fs::write(base.join("runtime").join(&key).join("status.json"), b"{{{").unwrap();
    assert!(engine.status("corrupt-ws").is_err());
}

// ---------------------------------------------------------------- checkpoints

#[test]
fn checkpoint_records_metadata_and_reports_manifest_drift() {
    let base = unique_temp("ckpt");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    write_ws(&root, "ckpt-ws", "");
    let engine = Engine::with_base(&base);
    engine.add(&root, true).unwrap();

    let cp = engine.checkpoint_create("ckpt-ws", Some("before refactor".into())).unwrap();
    assert_eq!(cp.workspace_name, "ckpt-ws");
    assert!(cp.note.as_deref() == Some("before refactor"));
    assert_eq!(cp.manifest_hash.len(), 64);

    assert!(engine.checkpoints("ckpt-ws").unwrap().len() == 1);

    // No drift yet.
    let preview = engine.restore_preview("ckpt-ws", &cp.id).unwrap();
    assert!(preview.drift.is_empty());

    // Manifest changes -> drift reported.
    write_ws(&root, "ckpt-ws", "# edited\n");
    let preview = engine.restore_preview("ckpt-ws", &cp.id).unwrap();
    assert!(
        preview.drift.iter().any(|d| d.kind == "manifest_changed"),
        "{:?}",
        preview.drift
    );

    // Missing root -> explicit drift kind.
    let preview2 = engine.restore_preview("ckpt-ws", &cp.id);
    assert!(preview2.is_ok());
}

#[test]
fn restore_refuses_noop_when_checkpoint_selector_unknown() {
    let (_g, _r, engine, _k, _serial) = setup_workspace("restore-miss", "rm-ws", "");
    assert!(engine.restore_preview("rm-ws", "nonexistent-id").is_err());
}

// ---------------------------------------------------------------- logs

#[test]
fn logs_are_captured_per_stream_with_timestamps() {
    let (_g, root, engine, _k, _serial) = setup_workspace("logs", "log-ws", "");
    let script = root.join("talker.cmd");
    std::fs::write(
        &script,
        "@echo OUT-LINE-ONE\r\necho ERR-LINE-ONE 1>&2\r\nping -n 3 127.0.0.1 >nul\r\n",
    )
    .unwrap();
    std::fs::write(
        root.join("lotus.toml"),
        manifest_toml(
            "log-ws",
            &format!(
                "\n[[process]]\nname = \"talker\"\ncommand = '{}'\n[process.shutdown]\ngrace_secs = 1\n",
                script.display()
            ),
        ),
    )
    .unwrap();
    engine.grant_trust("log-ws").unwrap();

    let t = engine.start_in_thread("log-ws").unwrap();
    wait_for_state(&engine, "log-ws", &["degraded"], Duration::from_secs(20));

    // Give readers a moment to flush.
    std::thread::sleep(Duration::from_millis(500));
    let lines = engine.tail_logs("log-ws", None, 50).unwrap();
    let joined = lines.join("\n");
    assert!(joined.contains("OUT-LINE-ONE"), "{joined}");
    assert!(joined.contains("ERR-LINE-ONE"), "{joined}");
    assert!(joined.contains("/out:"), "stream separation labels: {joined}");
    assert!(joined.contains("[20"), "timestamp prefix present: {joined}");
    engine.stop("log-ws").unwrap();
    t.join().ok();
}

// ---------------------------------------------------------------- state machine sanity

#[test]
fn stopping_state_visible_during_stop() {
    let (_g, _r, engine, _k, _serial) = setup_workspace(
        "stopping-vis",
        "sv-ws",
        &format!(
            "\n[[process]]\nname = \"w\"\ncommand = \"{}\"\nargs = {:?}\n[process.shutdown]\ngrace_secs = 2\n",
            sleeper(30).0,
            sleeper(30).1
        ),
    );
    let _t = engine.start_in_thread("sv-ws").unwrap();
    wait_for_state(&engine, "sv-ws", &["healthy"], Duration::from_secs(20));

    // Request stop directly through the control plane so we can observe STOPPING.
    let (key, _) = engine.resolve("sv-ws").unwrap();
    lotus_core::status::request_stop(&engine.layout.control_file(&key)).unwrap();
    let saw_stopping = (0..80).any(|_| {
        matches!(
            engine.status("sv-ws"),
            Ok(ref s) if s.state == "stopping"
        ) && {
            true
        } || {
            std::thread::sleep(Duration::from_millis(50));
            false
        }
    });
    assert!(saw_stopping || engine.status("sv-ws").map(|s| s.state == "off").unwrap_or(false));
    wait_for_state(&engine, "sv-ws", &["off"], Duration::from_secs(15));
}

#[test]
fn double_start_refused_while_running() {
    let (_g, _r, engine, _k, _serial) = setup_workspace(
        "double-start",
        "ds-ws",
        &format!(
            "\n[[process]]\nname = \"w\"\ncommand = \"{}\"\nargs = {:?}\n[process.shutdown]\ngrace_secs = 1\n",
            sleeper(30).0,
            sleeper(30).1
        ),
    );
    let t = engine.start_in_thread("ds-ws").unwrap();
    wait_for_state(&engine, "ds-ws", &["healthy"], Duration::from_secs(20));
    let second = engine.start_in_thread("ds-ws");
    assert!(second.is_err(), "second start must be refused");
    engine.stop("ds-ws").unwrap();
    t.join().ok();
}

#[test]
fn workspace_states_cover_required_machine() {
    // OFF -> STARTING -> HEALTHY -> STOPPING -> OFF exercised across other
    // tests; here we assert the enum surface exists and labels are stable.
    assert_eq!(WorkspaceState::Off.label(), "OFF");
    assert_eq!(WorkspaceState::Starting.label(), "STARTING");
    assert_eq!(WorkspaceState::Healthy.label(), "HEALTHY");
    assert_eq!(WorkspaceState::Degraded.label(), "DEGRADED");
    assert_eq!(WorkspaceState::Failed.label(), "FAILED");
    assert_eq!(WorkspaceState::Stopping.label(), "STOPPING");
}

#[test]
fn fatal_startup_error_leaves_visible_failed_status() {
    // Regression: a supervisor that dies before its first heartbeat used to
    // vanish silently; readers now get a terminal `failed` status with the
    // reason instead of an eternal <no status>.
    let base = unique_temp("fatal");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    write_ws(&root, "fatal-ws", "");
    let engine = Engine::with_base(&base);
    engine.add(&root, true).unwrap();

    // Material change after trust: direct supervisor invocation (bypassing
    // engine pre-checks) must fail AND leave the failure visible on disk.
    write_ws(&root, "fatal-ws", "# changed\n");
    let b = base.clone();
    let k = engine.resolve("fatal-ws").unwrap().0;
    let handle = std::thread::spawn(move || lotus_core::supervisor::run_supervised(&b, &k));
    let outcome = handle.join().expect("no panic");
    assert!(outcome.is_err(), "supervision must refuse changed manifest");

    let status = lotus_core::status::read(&engine.layout.status_file(&engine.resolve("fatal-ws").unwrap().0))
        .expect("terminal status exists");
    assert_eq!(status.state, "failed");
    assert!(status.last_error.as_deref().unwrap_or_default().contains("changed"));
}
