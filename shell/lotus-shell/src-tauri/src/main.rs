#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Tauri command boundary for LotusOS. Every command is a thin wrapper over
//! the same `lotus_core::engine::Engine` the CLI uses — there is exactly one
//! implementation of workspace behavior.

use std::sync::OnceLock;

use lotus_core::engine::{Engine, ListEntry, RestorePreview};
use lotus_core::manifest::Manifest;
use lotus_core::status::StatusReport;
use lotus_core::{checkpoint, doctor, ledger, LotusError};

static ENGINE: OnceLock<Engine> = OnceLock::new();

fn engine() -> &'static Engine {
    ENGINE.get_or_init(Engine::new)
}

fn err_msg(e: LotusError) -> String {
    e.to_string()
}

// ---------------------------------------------------------------- review/add

#[derive(serde::Serialize)]
pub struct ProcessReview {
    name: String,
    command: String,
    args: Vec<String>,
    ports: Vec<u16>,
}

#[derive(serde::Serialize)]
pub struct TrustReview {
    key: String,
    name: String,
    description: Option<String>,
    root: String,
    processes: Vec<ProcessReview>,
    env_required: Vec<String>,
    manifest_hash: String,
}

/// Inspect a manifest BEFORE any trust decision. Read-only.
#[tauri::command]
fn review_workspace(path: String) -> Result<TrustReview, String> {
    let root = std::path::PathBuf::from(&path);
    let (manifest, hash) = Manifest::load(&root.join("lotus.toml")).map_err(err_msg)?;
    Ok(TrustReview {
        key: lotus_core::trust::workspace_key(&root),
        name: manifest.name.clone(),
        description: manifest.description.clone(),
        root: manifest.root.display().to_string(),
        processes: manifest
            .processes
            .iter()
            .map(|p| ProcessReview {
                name: p.name.clone(),
                command: p.command.clone(),
                args: p.args.clone(),
                ports: p.ports.clone(),
            })
            .collect(),
        env_required: manifest.env_required.clone(),
        manifest_hash: hash,
    })
}

/// Register a workspace; `trust` records an explicit local trust decision.
#[tauri::command]
fn add_workspace(path: String, trust: bool) -> Result<String, String> {
    engine()
        .add(&std::path::PathBuf::from(&path), trust)
        .map_err(err_msg)
}

#[tauri::command]
fn grant_trust(selector: String) -> Result<(), String> {
    engine().grant_trust(&selector).map_err(err_msg)
}

#[tauri::command]
fn remove_workspace(selector: String) -> Result<(), String> {
    engine().remove(&selector).map_err(err_msg)
}

#[tauri::command]
fn list_workspaces() -> Vec<ListEntry> {
    engine().list()
}

// ---------------------------------------------------------------- lifecycle

#[tauri::command]
fn start_workspace(selector: String) -> Result<(), String> {
    engine().start_in_thread(&selector).map(|_| ()).map_err(err_msg)
}

#[tauri::command]
fn stop_workspace(selector: String) -> Result<Vec<String>, String> {
    engine().stop(&selector).map_err(err_msg)
}

#[tauri::command]
fn restart_workspace(selector: String) -> Result<(), String> {
    let e = engine();
    if e.status(&selector).map(|s| s.fresh() && s.state != "off").unwrap_or(false) {
        e.stop(&selector).map_err(err_msg)?;
    }
    e.start_in_thread(&selector).map(|_| ()).map_err(err_msg)
}

#[tauri::command]
fn workspace_status(selector: String) -> Result<StatusReport, String> {
    match engine().status(&selector) {
        Ok(s) => Ok(s),
        Err(_) => {
            // Never-started workspaces report OFF instead of an error so the
            // UI can render them normally.
            let (key, ws) = engine().resolve(&selector).map_err(err_msg)?;
            Ok(StatusReport {
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
    }
}

// ---------------------------------------------------------------- inspection

#[tauri::command]
fn doctor_workspace(selector: String) -> Result<Vec<doctor::Finding>, String> {
    engine().doctor(&selector).map_err(err_msg)
}

#[tauri::command]
fn workspace_events(selector: String, limit: u32) -> Result<Vec<ledger::Event>, String> {
    engine().events(&selector, limit as usize).map_err(err_msg)
}

#[tauri::command]
fn workspace_logs(
    selector: String,
    process: Option<String>,
    lines: u32,
) -> Result<Vec<String>, String> {
    engine()
        .tail_logs(&selector, process.as_deref(), lines as usize)
        .map_err(err_msg)
}

// ---------------------------------------------------------------- checkpoints

#[tauri::command]
fn create_checkpoint(selector: String, note: Option<String>) -> Result<checkpoint::Checkpoint, String> {
    engine().checkpoint_create(&selector, note).map_err(err_msg)
}

#[tauri::command]
fn list_checkpoints(selector: String) -> Result<Vec<checkpoint::Checkpoint>, String> {
    engine().checkpoints(&selector).map_err(err_msg)
}

#[tauri::command]
fn restore_preview(
    selector: String,
    checkpoint_id: String,
) -> Result<RestorePreview, String> {
    engine().restore_preview(&selector, &checkpoint_id).map_err(err_msg)
}

#[tauri::command]
fn restore_workspace(selector: String, checkpoint_id: String) -> Result<Vec<checkpoint::Drift>, String> {
    let preview = engine().restore_preview(&selector, &checkpoint_id).map_err(err_msg)?;
    engine().stop(&selector).map_err(err_msg)?;
    engine().start_in_thread(&selector).map_err(err_msg)?;
    Ok(preview.drift)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            review_workspace,
            add_workspace,
            grant_trust,
            remove_workspace,
            list_workspaces,
            start_workspace,
            stop_workspace,
            restart_workspace,
            workspace_status,
            doctor_workspace,
            workspace_events,
            workspace_logs,
            create_checkpoint,
            list_checkpoints,
            restore_preview,
            restore_workspace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lotus Shell");
}
