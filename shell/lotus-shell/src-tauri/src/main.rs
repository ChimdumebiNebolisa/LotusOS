#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const UNKNOWN_VALUE: &str = "Unknown";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemSnapshot {
    lotus_name: String,
    lotus_pretty_name: String,
    lotus_stage: String,
    os_name: String,
    os_pretty_name: String,
    base_id: String,
    version_codename: String,
    username: String,
    hostname: String,
    session_mode: String,
    session_type: String,
    current_desktop: String,
    desktop_session: String,
    display_protocol: String,
    has_calamares_launcher: bool,
}

#[derive(Clone, Copy)]
struct AppDefinition {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    command_candidates: &'static [&'static str],
    args: &'static [&'static str],
    live_only: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalApp {
    id: String,
    label: String,
    description: String,
    available: bool,
    visible: bool,
}

#[derive(Clone, Copy)]
struct ResourceDefinition {
    id: &'static str,
    section_id: &'static str,
    label: &'static str,
    description: &'static str,
    candidates: &'static [&'static str],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalResource {
    id: String,
    section_id: String,
    label: String,
    description: String,
    path: String,
    exists: bool,
}

const APP_DEFINITIONS: [AppDefinition; 7] = [
    AppDefinition {
        id: "terminal",
        label: "Terminal",
        description: "Open Konsole for shell access and local commands.",
        command_candidates: &["konsole"],
        args: &[],
        live_only: false,
    },
    AppDefinition {
        id: "files",
        label: "Files",
        description: "Open Dolphin to browse local folders and working files.",
        command_candidates: &["dolphin"],
        args: &[],
        live_only: false,
    },
    AppDefinition {
        id: "browser",
        label: "Browser",
        description: "Open Firefox ESR for local docs and web access.",
        command_candidates: &["firefox-esr", "firefox"],
        args: &[],
        live_only: false,
    },
    AppDefinition {
        id: "editor",
        label: "Editor",
        description: "Open Kate for notes, configs, and local editing.",
        command_candidates: &["kate"],
        args: &[],
        live_only: false,
    },
    AppDefinition {
        id: "pdf",
        label: "PDF",
        description: "Open Okular for PDFs and reference documents.",
        command_candidates: &["okular"],
        args: &[],
        live_only: false,
    },
    AppDefinition {
        id: "office",
        label: "Office",
        description: "Open LibreOffice for documents, sheets, and slides.",
        command_candidates: &["libreoffice"],
        args: &[],
        live_only: false,
    },
    AppDefinition {
        id: "installer",
        label: "Install LotusOS",
        description: "Open the Calamares installer when this is a live session.",
        command_candidates: &["calamares-install-debian"],
        args: &[],
        live_only: true,
    },
];

const RESOURCE_DEFINITIONS: [ResourceDefinition; 9] = [
    ResourceDefinition {
        id: "projects-home",
        section_id: "projects",
        label: "Home Workspace",
        description: "Anchor project work to the current user's home directory.",
        candidates: &[""],
    },
    ResourceDefinition {
        id: "projects-projects",
        section_id: "projects",
        label: "Projects",
        description: "Use the conventional Projects folder if it exists.",
        candidates: &["Projects", "projects"],
    },
    ResourceDefinition {
        id: "projects-code",
        section_id: "projects",
        label: "Code",
        description: "Use a dedicated code or source folder when present.",
        candidates: &["Code", "code", "src", "Workspace"],
    },
    ResourceDefinition {
        id: "notes-notes",
        section_id: "notes",
        label: "Notes Folder",
        description: "Prefer an explicit Notes folder when one already exists.",
        candidates: &["Notes", "notes", "Documents/Notes"],
    },
    ResourceDefinition {
        id: "notes-documents",
        section_id: "notes",
        label: "Documents",
        description: "Fall back to the main documents folder for drafts and references.",
        candidates: &["Documents"],
    },
    ResourceDefinition {
        id: "files-home",
        section_id: "files",
        label: "Home",
        description: "The current user's home directory.",
        candidates: &[""],
    },
    ResourceDefinition {
        id: "files-desktop",
        section_id: "files",
        label: "Desktop",
        description: "Quick access to desktop files and scratch material.",
        candidates: &["Desktop"],
    },
    ResourceDefinition {
        id: "files-documents",
        section_id: "files",
        label: "Documents",
        description: "Working documents and longer-form references.",
        candidates: &["Documents"],
    },
    ResourceDefinition {
        id: "files-downloads",
        section_id: "files",
        label: "Downloads",
        description: "Recent downloads and imported resources.",
        candidates: &["Downloads"],
    },
];

#[tauri::command]
fn get_system_snapshot() -> SystemSnapshot {
    system_snapshot()
}

#[tauri::command]
fn get_local_apps() -> Vec<LocalApp> {
    let snapshot = system_snapshot();

    APP_DEFINITIONS
        .iter()
        .filter_map(|definition| {
            let visible = !definition.live_only
                || (snapshot.session_mode == "live" && snapshot.has_calamares_launcher);

            if !visible {
                return None;
            }

            Some(LocalApp {
                id: definition.id.to_string(),
                label: definition.label.to_string(),
                description: definition.description.to_string(),
                available: resolve_command(definition.command_candidates).is_some(),
                visible,
            })
        })
        .collect()
}

#[tauri::command]
fn launch_local_app(app_id: String) -> Result<(), String> {
    let snapshot = system_snapshot();
    let Some(definition) = APP_DEFINITIONS.iter().find(|definition| definition.id == app_id) else {
        return Err("Unknown launcher.".to_string());
    };

    if definition.live_only && !(snapshot.session_mode == "live" && snapshot.has_calamares_launcher)
    {
        return Err("This launcher is only available in a live LotusOS session.".to_string());
    }

    let Some(command_path) = resolve_command(definition.command_candidates) else {
        return Err(format!("{} is not available in this session.", definition.label));
    };

    Command::new(command_path)
        .args(definition.args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Failed to launch {}: {}", definition.label, error))
}

#[tauri::command]
fn get_local_resources() -> Vec<LocalResource> {
    let home = home_dir();

    RESOURCE_DEFINITIONS
        .iter()
        .filter_map(|definition| {
            let home = home.as_ref()?;
            let resolved_path = resolve_resource_path(home, definition)?;

            Some(LocalResource {
                id: definition.id.to_string(),
                section_id: definition.section_id.to_string(),
                label: definition.label.to_string(),
                description: definition.description.to_string(),
                path: resolved_path.display().to_string(),
                exists: resolved_path.exists(),
            })
        })
        .collect()
}

#[tauri::command]
fn open_local_resource(resource_id: String) -> Result<(), String> {
    let Some(home) = home_dir() else {
        return Err("Could not resolve the current home directory.".to_string());
    };

    let Some(definition) = RESOURCE_DEFINITIONS
        .iter()
        .find(|definition| definition.id == resource_id) else {
        return Err("Unknown local resource.".to_string());
    };

    let Some(path) = resolve_resource_path(&home, definition) else {
        return Err("Could not resolve that local resource path.".to_string());
    };

    if !path.exists() {
        return Err("That local resource path does not exist yet.".to_string());
    }

    let Some(file_manager) = resolve_command(&["dolphin"]) else {
        return Err("Dolphin is not available in this session.".to_string());
    };

    Command::new(file_manager)
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Failed to open the local resource: {}", error))
}

fn parse_release_file(path: &str) -> HashMap<String, String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return HashMap::new();
    };

    contents
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }

            let (key, value) = trimmed.split_once('=')?;
            Some((key.trim().to_string(), strip_quotes(value.trim()).to_string()))
        })
        .collect()
}

fn strip_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|trimmed| trimmed.strip_suffix('"'))
        .unwrap_or(value)
}

fn release_value(release_map: &HashMap<String, String>, key: &str) -> String {
    release_map
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| UNKNOWN_VALUE.to_string())
}

fn first_non_empty<'a>(values: impl IntoIterator<Item = Option<&'a str>>) -> String {
    values
        .into_iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
        .unwrap_or(UNKNOWN_VALUE)
        .to_string()
}

fn env_value(key: &str) -> String {
    env_value_opt(key).unwrap_or_else(|| UNKNOWN_VALUE.to_string())
}

fn env_value_opt(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn username() -> String {
    env_value_opt("USER")
        .or_else(|| env_value_opt("LOGNAME"))
        .or_else(|| env_value_opt("USERNAME"))
        .unwrap_or_else(|| UNKNOWN_VALUE.to_string())
}

fn hostname() -> String {
    env_value_opt("HOSTNAME")
        .or_else(|| read_trimmed_file("/etc/hostname"))
        .unwrap_or_else(|| UNKNOWN_VALUE.to_string())
}

fn read_trimmed_file(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|contents| contents.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn detect_live_session() -> bool {
    Path::new("/run/live/medium").exists()
        || Path::new("/lib/live/mount/medium").exists()
        || read_trimmed_file("/proc/cmdline")
            .map(|cmdline| cmdline.contains("boot=live"))
            .unwrap_or(false)
}

fn display_protocol() -> String {
    if env::var("WAYLAND_DISPLAY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .is_some()
    {
        return "Wayland".to_string();
    }

    if env::var("DISPLAY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .is_some()
    {
        return "X11".to_string();
    }

    UNKNOWN_VALUE.to_string()
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("USERPROFILE")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
        })
}

fn system_snapshot() -> SystemSnapshot {
    let lotus_release = parse_release_file("/etc/lotusos-release");
    let os_release = parse_release_file("/etc/os-release");
    let is_live_session = detect_live_session();

    SystemSnapshot {
        lotus_name: release_value(&lotus_release, "NAME"),
        lotus_pretty_name: release_value(&lotus_release, "PRETTY_NAME"),
        lotus_stage: release_value(&lotus_release, "LOTUSOS_STAGE"),
        os_name: release_value(&os_release, "NAME"),
        os_pretty_name: release_value(&os_release, "PRETTY_NAME"),
        base_id: first_non_empty([
            lotus_release.get("BASE_ID").map(String::as_str),
            os_release.get("ID").map(String::as_str),
        ]),
        version_codename: release_value(&os_release, "VERSION_CODENAME"),
        username: username(),
        hostname: hostname(),
        session_mode: if is_live_session {
            "live".to_string()
        } else {
            "installed".to_string()
        },
        session_type: env_value("XDG_SESSION_TYPE"),
        current_desktop: env_value("XDG_CURRENT_DESKTOP"),
        desktop_session: env_value("DESKTOP_SESSION"),
        display_protocol: display_protocol(),
        has_calamares_launcher: Path::new("/usr/share/applications/calamares-install-debian.desktop")
            .exists(),
    }
}

fn resolve_command(candidates: &[&str]) -> Option<PathBuf> {
    candidates.iter().find_map(|candidate| {
        resolve_absolute_command(candidate).or_else(|| resolve_command_on_path(candidate))
    })
}

fn resolve_absolute_command(candidate: &str) -> Option<PathBuf> {
    let path = PathBuf::from(candidate);

    if path.is_absolute() && path.exists() {
        return Some(path);
    }

    None
}

fn resolve_command_on_path(candidate: &str) -> Option<PathBuf> {
    let path_value = env::var_os("PATH")?;

    env::split_paths(&path_value)
        .map(|directory| directory.join(candidate))
        .find(|path| path.exists())
}

fn resolve_resource_path(home: &Path, definition: &ResourceDefinition) -> Option<PathBuf> {
    definition
        .candidates
        .iter()
        .map(|candidate| {
            if candidate.is_empty() {
                home.to_path_buf()
            } else {
                home.join(candidate)
            }
        })
        .find(|path| path.exists())
        .or_else(|| {
            definition.candidates.first().map(|candidate| {
                if candidate.is_empty() {
                    home.to_path_buf()
                } else {
                    home.join(candidate)
                }
            })
        })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_system_snapshot,
            get_local_apps,
            launch_local_app,
            get_local_resources,
            open_local_resource
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lotus Shell");
}
