use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use lotus_core::engine::Engine;
use lotus_core::manifest::Manifest;
use lotus_core::LotusError;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let engine = Engine::new();
    match dispatch(&engine, &args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            match err {
                LotusError::NotFound(_) => ExitCode::from(1),
                _ => ExitCode::from(1),
            }
        }
    }
}

const USAGE: &str = "\
lotus - operate a development workspace as one unit

USAGE:
  lotus add <path> [--trust]     register a workspace (trust decision required to start)
  lotus list                     show registered workspaces
  lotus trust <workspace>        grant/re-grant trust after reviewing the manifest
  lotus start <workspace>        start all processes in dependency order
  lotus stop <workspace>         graceful stop (forced only after declared grace)
  lotus restart <workspace>      stop then start
  lotus status <workspace>       current lifecycle state
  lotus doctor <workspace>       environment + manifest diagnostics
  lotus logs <workspace> [--process NAME] [--lines N]
  lotus events <workspace> [--limit N]   local lifecycle event ledger
  lotus checkpoint <workspace> [--note TEXT]
  lotus checkpoints <workspace>          list checkpoints
  lotus restore <workspace> <checkpoint-id>
";

fn dispatch(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    let Some(cmd) = args.first() else {
        println!("{USAGE}");
        return Ok(());
    };

    let rest = &args[1..];
    match cmd.as_str() {
        "add" => cmd_add(engine, rest),
        "list" => cmd_list(engine),
        "trust" => cmd_trust(engine, rest),
        "start" => cmd_start(engine, rest),
        "stop" => cmd_stop(engine, rest),
        "restart" => cmd_restart(engine, rest),
        "status" => cmd_status(engine, rest),
        "doctor" => cmd_doctor(engine, rest),
        "logs" => cmd_logs(engine, rest),
        "events" => cmd_events(engine, rest),
        "checkpoint" => cmd_checkpoint(engine, rest),
        "checkpoints" => cmd_checkpoints(engine, rest),
        "restore" => cmd_restore(engine, rest),
        "__supervise" if rest.len() == 1 => {
            lotus_core::supervisor::run_supervised(&engine.base, &rest[0])
        }
        "-h" | "--help" | "help" => {
            println!("{USAGE}");
            Ok(())
        }
        other => {
            eprintln!("unknown command `{other}`\n");
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    }
}

// ---------------------------------------------------------------- add

fn cmd_add(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    let mut path: Option<String> = None;
    let mut trust = false;
    for arg in args {
        match arg.as_str() {
            "--trust" => trust = true,
            other if path.is_none() => path = Some(other.to_string()),
            other => return usage_err(format!("unexpected argument `{other}`")),
        }
    }
    let Some(path) = path else { return usage_err("add requires <path>".into()) };
    let root = PathBuf::from(path);
    if !root.is_dir() {
        return Err(LotusError::NotFound(format!(
            "{} is not a directory",
            root.display()
        )));
    }

    // Parse first so the trust decision is based on validated content.
    let (manifest, _hash) =
        Manifest::load(&root.join("lotus.toml")).map_err(LotusError::from)?;
    print_trust_summary(&manifest);

    let granted = if trust {
        true
    } else {
        prompt_trust()?
    };
    engine.add(&root, granted)?;
    if granted {
        println!("workspace added and trusted.");
    } else {
        println!("workspace added WITHOUT trust; `lotus start` will refuse until you run `lotus trust`.");
    }
    Ok(())
}

fn print_trust_summary(manifest: &Manifest) {
    println!("workspace : {}", manifest.name);
    if let Some(desc) = &manifest.description {
        println!("about     : {desc}");
    }
    println!("root      : {}", manifest.root.display());
    println!("processes : {}", manifest.processes.len());
    for p in &manifest.processes {
        println!(
            "  {:<14} {} {}{}",
            p.name,
            p.command,
            p.args.join(" "),
            if p.ports.is_empty() {
                String::new()
            } else {
                format!("  (ports: {:?})", p.ports)
            }
        );
    }
    if !manifest.env_required.is_empty() {
        println!(
            "requires env vars (names only): {}",
            manifest.env_required.join(", ")
        );
    }
    println!("manifest hash: {}", &manifest.hash[..16.min(manifest.hash.len())]);
}

fn prompt_trust() -> Result<bool, LotusError> {
    print!("\nTrust this workspace and allow its commands to run? [y/N] ");
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|e| LotusError::State(format!("stdin read failed: {e}")))?;
    Ok(matches!(line.trim(), "y" | "Y" | "yes"))
}

// ---------------------------------------------------------------- list

fn cmd_list(engine: &Engine) -> Result<(), LotusError> {
    let entries = engine.list();
    if entries.is_empty() {
        println!("no workspaces registered. try: lotus add <path>");
        return Ok(());
    }
    println!(
        "{:<18} {:<10} {:<6} {:<8} ROOT",
        "NAME", "STATE", "TRUST", "SUPERVISOR"
    );
    for e in entries {
        println!(
            "{:<18} {:<10} {:<6} {:<8} {}",
            e.name,
            e.state,
            if e.trusted { "yes" } else { "no" },
            if e.supervisor_alive { "live" } else { "-" },
            e.root
        );
    }
    Ok(())
}

// ---------------------------------------------------------------- trust

fn cmd_trust(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    let Some(selector) = single(args)? else {
        return usage_err("trust requires <workspace>".into());
    };
    let (_, manifest) = engine.manifest_for_review(&selector)?;
    print_trust_summary(&manifest);
    if prompt_trust()? {
        engine.grant_trust(&selector)?;
        println!("trust granted.");
        Ok(())
    } else {
        println!("trust NOT granted.");
        Ok(())
    }
}

// ---------------------------------------------------------------- lifecycle

fn cmd_start(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    let Some(selector) = single(args)? else {
        return usage_err("start requires <workspace>".into());
    };
    engine.start_detached(&selector)?;
    println!("start requested.");
    // Show initial status transition.
    std::thread::sleep(std::time::Duration::from_millis(700));
    match engine.status(&selector) {
        Ok(s) => println!("state: {} ({})", s.state, s.name),
        Err(_) => println!("supervisor is starting; run `lotus status` in a moment"),
    }
    Ok(())
}

fn cmd_stop(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    let Some(selector) = single(args)? else {
        return usage_err("stop requires <workspace>".into());
    };
    let notes = engine.stop(&selector)?;
    println!("stop complete.");
    for n in notes {
        println!("note: {n}");
    }
    Ok(())
}

fn cmd_restart(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    let Some(selector) = single(args)? else {
        return usage_err("restart requires <workspace>".into());
    };
    engine.restart(&selector)?;
    println!("restart requested.");
    Ok(())
}

fn cmd_status(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    let Some(selector) = single(args)? else {
        return usage_err("status requires <workspace>".into());
    };
    let s = engine.status(&selector).or_else(|_| fallback_off(engine, &selector))?;
    println!("workspace : {} ({})", s.name, s.root);
    println!("state     : {}", s.state);
    if let Some(started) = s.started_at_ms {
        println!("started   : {}", lotus_core::util::format_ts(started));
    }
    if !s.port_conflicts.is_empty() {
        println!("port conflicts:");
        for c in &s.port_conflicts {
            println!(
                "  port {}: wanted by `{}`, held by {} (pid {}) - {}",
                c.port,
                c.expected_owner,
                c.owner_name.as_deref().unwrap_or("?"),
                c.owner_pid.map(|p| p.to_string()).unwrap_or_else(|| "?".into()),
                c.remediation
            );
        }
    }
    if let Some(err) = &s.last_error {
        println!("last error: {err}");
    }
    if !s.processes.is_empty() {
        println!(
            "{:<16} {:<11} {:<9} {:<5} RESTARTS",
            "PROCESS", "STATE", "HEALTH", "PID"
        );
        for p in &s.processes {
            println!(
                "{:<16} {:<11} {:<9} {:<5} {}",
                p.name,
                p.state,
                match p.healthy {
                    Some(true) => "pass",
                    Some(false) => "FAIL",
                    None => "-",
                },
                p.pid.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
                p.restarts
            );
            if let (Some(false), Some(detail)) = (p.healthy, &p.detail) {
                println!("  └─ {detail}");
            }
        }
    }
    Ok(())
}

fn fallback_off(engine: &Engine, selector: &str) -> Result<lotus_core::status::StatusReport, LotusError> {
    let (key, ws) = engine.resolve(selector)?;
    Ok(lotus_core::status::StatusReport {
        key,
        name: ws.name,
        root: ws.root,
        manifest_hash: String::new(),
        state: "off".into(),
        started_at_ms: None,
        updated_at_ms: 0,
        processes: vec![],
        port_conflicts: vec![],
        last_error: None,
    })
}

// ---------------------------------------------------------------- doctor

fn cmd_doctor(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    let Some(selector) = single(args)? else {
        return usage_err("doctor requires <workspace>".into());
    };
    let findings = engine.doctor(&selector)?;
    let bad = findings
        .iter()
        .any(|f| f.status != lotus_core::doctor::FindingStatus::Ok);
    println!(
        "{:<12} {:<24} {:<10} MESSAGE",
        "CHECK", "SUBJECT", "STATUS"
    );
    for f in &findings {
        println!(
            "{:<12} {:<24} {:<10} {}",
            f.check,
            truncate(&f.subject, 24),
            format!("{:?}", f.status).to_uppercase(),
            f.message
        );
    }
    println!(
        "\ndoctor result: {}",
        if bad { "PROBLEMS FOUND" } else { "ALL OK" }
    );
    if bad {
        std::process::exit(1);
    }
    Ok(())
}

// ---------------------------------------------------------------- logs/events

fn cmd_logs(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    let mut selector: Option<String> = None;
    let mut process: Option<String> = None;
    let mut lines: usize = 40;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--process" if i + 1 < args.len() => {
                process = Some(args[i + 1].clone());
                i += 1;
            }
            "--lines" if i + 1 < args.len() => {
                lines = args[i + 1].parse().map_err(|_| {
                    LotusError::State("--lines expects a number".into())
                })?;
                i += 1;
            }
            other if selector.is_none() => selector = Some(other.to_string()),
            other => return usage_err(format!("unexpected argument `{other}`")),
        }
        i += 1;
    }
    let Some(selector) = selector else {
        return usage_err("logs requires <workspace>".into());
    };
    for line in engine.tail_logs(&selector, process.as_deref(), lines)? {
        println!("{line}");
    }
    Ok(())
}

fn cmd_events(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    let mut selector: Option<String> = None;
    let mut limit: usize = 30;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--limit" if i + 1 < args.len() => {
                limit = args[i + 1].parse().map_err(|_| {
                    LotusError::State("--limit expects a number".into())
                })?;
                i += 1;
            }
            other if selector.is_none() => selector = Some(other.to_string()),
            other => return usage_err(format!("unexpected argument `{other}`")),
        }
        i += 1;
    }
    let Some(selector) = selector else {
        return usage_err("events requires <workspace>".into());
    };
    for e in engine.events(&selector, limit)? {
        println!(
            "#{:04} {} {}{}",
            e.seq,
            e.ts_display,
            e.kind,
            e.process
                .as_ref()
                .map(|p| format!(" [{p}]"))
                .unwrap_or_default(),
        );
    }
    Ok(())
}

// ---------------------------------------------------------------- checkpoints

fn cmd_checkpoint(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    let mut selector: Option<String> = None;
    let mut note: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--note" if i + 1 < args.len() => {
                note = Some(args[i + 1].clone());
                i += 1;
            }
            other if selector.is_none() => selector = Some(other.to_string()),
            other => return usage_err(format!("unexpected argument `{other}`")),
        }
        i += 1;
    }
    let Some(selector) = selector else {
        return usage_err("checkpoint requires <workspace>".into());
    };
    let cp = engine.checkpoint_create(&selector, note)?;
    println!("checkpoint {} saved ({}).", cp.id, cp.created_at);
    Ok(())
}

fn cmd_checkpoints(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    let Some(selector) = single(args)? else {
        return usage_err("checkpoints requires <workspace>".into());
    };
    let list = engine.checkpoints(&selector)?;
    if list.is_empty() {
        println!("no checkpoints yet. try: lotus checkpoint <workspace>");
        return Ok(());
    }
    for c in list {
        println!(
            "{}  {}  branch={} commit={} note={}",
            c.id,
            c.created_at,
            c.git_branch.as_deref().unwrap_or("-"),
            c.git_commit.as_deref().and_then(|s| s.get(..8)).unwrap_or("-"),
            c.note.as_deref().unwrap_or("-")
        );
    }
    Ok(())
}

fn cmd_restore(engine: &Engine, args: &[String]) -> Result<(), LotusError> {
    if args.len() < 2 {
        return usage_err("restore requires <workspace> <checkpoint-id>".into());
    }
    let selector = args[0].clone();
    let cp_selector = args[1].clone();

    let preview = engine.restore_preview(&selector, &cp_selector)?;
    let cp = &preview.checkpoint;
    println!(
        "restore target: checkpoint {} from {}",
        cp.id, cp.created_at
    );

    if preview.drift.is_empty() {
        println!("drift: none");
    } else {
        println!("drift detected:");
        for d in &preview.drift {
            println!("  {:<22} expected `{}`, found `{}`", d.kind, d.expected, d.found);
        }
    }

    engine.stop(&selector)?;
    engine.start_detached(&selector)?;
    println!("restore requested; inspect with `lotus status`.");
    Ok(())
}

// ---------------------------------------------------------------- helpers

fn single(args: &[String]) -> Result<Option<String>, LotusError> {
    match args.len() {
        0 => Ok(None),
        1 => Ok(Some(args[0].clone())),
        _ => usage_err(format!("expected one argument, got {:?}", args)),
    }
}

fn usage_err<T>(msg: String) -> Result<T, LotusError> {
    eprintln!("{msg}\n");
    eprintln!("{USAGE}");
    std::process::exit(2);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
