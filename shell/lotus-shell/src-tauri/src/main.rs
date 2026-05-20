#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::{collections::HashMap, env, fs, path::Path};

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

#[tauri::command]
fn get_system_snapshot() -> SystemSnapshot {
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_system_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running Lotus Shell");
}
