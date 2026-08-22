//! Environment diagnostics ("doctor").
//!
//! Doctor distinguishes Missing / Invalid / Unverified / Conflict from Ok and
//! NEVER prints environment variable values — only names and presence.

use std::path::Path;

use serde::Serialize;

use crate::error::Result;
use crate::gitctx;
use crate::manifest::Manifest;
use crate::platform;
use crate::ports;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Ok,
    Missing,
    Invalid,
    Unverified,
    Conflict,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub check: String,
    pub subject: String,
    pub status: FindingStatus,
    pub message: String,
}

fn finding(
    check: &str,
    subject: &str,
    status: FindingStatus,
    message: impl Into<String>,
) -> Finding {
    Finding {
        check: check.to_string(),
        subject: subject.to_string(),
        status,
        message: message.into(),
    }
}

/// Run every diagnostic the manifest declares. Read-only; never mutates the
/// workspace and never reveals secret values.
pub fn run(manifest: &Manifest) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    let root = &manifest.root;

    // Executables + optional runtime versions.
    for proc in &manifest.processes {
        match platform::resolve_executable(&proc.command) {
            Ok(path) => {
                findings.push(finding(
                    "executable",
                    &proc.name,
                    FindingStatus::Ok,
                    format!("`{}` resolved to {}", proc.command, path.display()),
                ));
                if let Some(version) = &proc.version {
                    findings.push(check_version(&proc.name, &path, version, root));
                }
            }
            Err(e) => findings.push(finding(
                "executable",
                &proc.name,
                FindingStatus::Missing,
                e.to_string(),
            )),
        }

        if let Some(env_file) = &proc.env_file {
            findings.push(finding(
                "env_file",
                &format!("{} ({})", proc.name, env_file.display()),
                if env_file.exists() {
                    FindingStatus::Ok
                } else {
                    FindingStatus::Missing
                },
                if env_file.exists() {
                    "present"
                } else {
                    "declared but missing"
                },
            ));
        }
    }

    // Required environment variables: presence only, never values.
    for var in &manifest.env_required {
        let present = std::env::var_os(var).is_some_and(|v| !v.is_empty());
        findings.push(finding(
            "env_var",
            var,
            if present { FindingStatus::Ok } else { FindingStatus::Missing },
            if present {
                "set (value not displayed)".to_string()
            } else {
                "not set".to_string()
            },
        ));
    }

    // Declared path checks: existence + writability for directories.
    for path in &manifest.path_checks {
        if !path.exists() {
            findings.push(finding(
                "path",
                &path.display().to_string(),
                FindingStatus::Missing,
                "does not exist",
            ));
            continue;
        }
        let writable = writable_dir(path);
        findings.push(finding(
            "path",
            &path.display().to_string(),
            if writable == Some(false) {
                FindingStatus::Invalid
            } else {
                FindingStatus::Ok
            },
            match writable {
                Some(true) => "exists, directory writable",
                Some(false) => "exists but not writable",
                None => "exists (file)",
            },
        ));
    }

    // Ports: report conflicts with remediation advice.
    let expected: Vec<(u16, String)> = manifest
        .processes
        .iter()
        .flat_map(|p| p.ports.iter().map(|port| (*port, p.name.clone())))
        .collect();
    match ports::detect_conflicts(&expected, &[]) {
        Ok(conflicts) => {
            let conflicted: std::collections::BTreeSet<u16> =
                conflicts.iter().map(|c| c.port).collect();
            for (port, owner) in &expected {
                if conflicted.contains(port) {
                    let c = conflicts.iter().find(|c| c.port == *port).expect("known");
                    findings.push(Finding {
                        check: "port".into(),
                        subject: port.to_string(),
                        status: FindingStatus::Conflict,
                        message: format!(
                            "wanted by `{owner}`; held by {} (pid {}) - {}",
                            c.owner_name.as_deref().unwrap_or("unknown process"),
                            c.owner_pid.map(|p| p.to_string()).unwrap_or_else(|| "?".into()),
                            c.remediation
                        ),
                    });
                } else {
                    findings.push(finding(
                        "port",
                        &port.to_string(),
                        FindingStatus::Ok,
                        format!("free for `{owner}`"),
                    ));
                }
            }
        }
        Err(_) => findings.push(finding(
            "port",
            "discovery",
            FindingStatus::Unverified,
            "listener discovery unavailable; on Unix install `lsof` or `ss`".to_string(),
        )),
    }

    // Git expectations (local reads only).
    if manifest.git_required || manifest.git_branch.is_some() {
        let ctx = gitctx::context(root);
        if !ctx.is_repo {
            findings.push(finding(
                "git",
                &root.display().to_string(),
                FindingStatus::Missing,
                "not a git repository",
            ));
        } else {
            findings.push(finding(
                "git",
                "repository",
                FindingStatus::Ok,
                format!(
                    "branch {} commit {} dirty={:?}",
                    ctx.branch.as_deref().unwrap_or("(detached)"),
                    ctx.commit.as_deref().and_then(|c| c.get(..12)).unwrap_or("?"),
                    ctx.dirty
                ),
            ));
            if let Some(expected_branch) = &manifest.git_branch {
                let matches = ctx.branch.as_deref() == Some(expected_branch.as_str());
                findings.push(finding(
                    "git_branch",
                    expected_branch,
                    if matches { FindingStatus::Ok } else { FindingStatus::Invalid },
                    if matches {
                        "on expected branch".into()
                    } else {
                        format!("currently on {}", ctx.branch.as_deref().unwrap_or("(detached HEAD)"))
                    },
                ));
            }
        }
    }

    Ok(findings)
}

fn check_version(
    process_name: &str,
    exe: &Path,
    spec: &crate::manifest::VersionSpec,
    cwd: &Path,
) -> Finding {
    let output = std::process::Command::new(exe)
        .args(&spec.args)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output();
    match output {
        Ok(out) => {
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            let first_line = text.lines().next().unwrap_or("").trim().to_string();
            match &spec.contains {
                Some(needle) => {
                    if text.contains(needle) {
                        finding(
                            "version",
                            process_name,
                            FindingStatus::Ok,
                            first_line,
                        )
                    } else {
                        finding(
                            "version",
                            process_name,
                            FindingStatus::Invalid,
                            format!("expected `{needle}` in version output, got: {first_line}"),
                        )
                    }
                }
                None => finding("version", process_name, FindingStatus::Ok, first_line),
            }
        }
        Err(_) => finding(
            "version",
            process_name,
            FindingStatus::Unverified,
            "could not execute version probe",
        ),
    }
}

fn writable_dir(path: &Path) -> Option<bool> {
    if !path.is_dir() {
        return None;
    }
    let probe = path.join(format!(".lotus-write-probe-{}", std::process::id()));
    match std::fs::write(&probe, b"ok") {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            Some(true)
        }
        Err(_) => Some(false),
    }
}
