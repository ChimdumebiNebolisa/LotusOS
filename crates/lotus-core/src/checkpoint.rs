//! Workspace checkpoints.
//!
//! A checkpoint is honest metadata, not a memory snapshot: it records the
//! manifest hash, git position, process set, declared paths, and the last
//! observed health so a workspace lifecycle can be reconstructed later while
//! reporting drift against the current filesystem.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{LotusError, Result};
use crate::gitctx;
use crate::manifest::Manifest;
use crate::paths::Paths;
use crate::util::{format_ts, now_ms, sha256_hex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub schema_version: u32,
    pub id: String,
    pub created_at_ms: u64,
    pub created_at: String,
    pub workspace_key: String,
    pub workspace_name: String,
    pub root: String,
    pub manifest_hash: String,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub git_dirty: Option<bool>,
    pub processes: Vec<String>,
    pub ports: Vec<u16>,
    /// Last known workspace state when taken (off/healthy/degraded/...).
    pub last_state: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Drift {
    pub kind: String,
    pub expected: String,
    pub found: String,
}

const SCHEMA_VERSION: u32 = 1;

pub fn checkpoint_dir(base: &Path, key: &str) -> PathBuf {
    base.join("checkpoints").join(key)
}

pub fn create(layout: &Paths, key: &str, manifest: &Manifest, last_state: Option<String>, note: Option<String>) -> Result<Checkpoint> {
    let git_ctx = gitctx::context(&manifest.root);
    let ts = now_ms();
    let id = format!(
        "{ts}-{}",
        sha256_hex(format!("{key}{ts}").as_bytes()).get(..8).unwrap_or("?")
    );
    let cp = Checkpoint {
        schema_version: SCHEMA_VERSION,
        id,
        created_at_ms: ts,
        created_at: format_ts(ts),
        workspace_key: key.to_string(),
        workspace_name: manifest.name.clone(),
        root: manifest.root.to_string_lossy().to_string(),
        manifest_hash: manifest.hash.clone(),
        git_branch: git_ctx.branch,
        git_commit: git_ctx.commit.map(|c| c.get(..40).unwrap_or("").to_string()),
        git_dirty: git_ctx.dirty,
        processes: manifest.processes.iter().map(|p| p.name.clone()).collect(),
        ports: manifest
            .processes
            .iter()
            .flat_map(|p| p.ports.iter().copied())
            .collect(),
        last_state,
        note,
    };

    let dir = checkpoint_dir(&layout.base, key);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(format!("{}.json", cp.id)), serde_json::to_vec_pretty(&cp)?)?;
    Ok(cp)
}

pub fn list(layout: &Paths, key: &str) -> Vec<Checkpoint> {
    let mut out: Vec<Checkpoint> = std::fs::read_dir(checkpoint_dir(&layout.base, key))
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
                .filter_map(|e| std::fs::read(e.path()).ok())
                .filter_map(|bytes| serde_json::from_slice(&bytes).ok())
                .collect()
        })
        .unwrap_or_default();
    out.sort_by_key(|c| std::cmp::Reverse(c.created_at_ms));
    out
}

/// Load one checkpoint by id (or unique id prefix) for this workspace.
pub fn find(layout: &Paths, key: &str, selector: &str) -> Result<Checkpoint> {
    if selector.contains('-') && !selector.is_empty() {
        let direct = checkpoint_dir(&layout.base, key).join(format!("{selector}.json"));
        if let Ok(bytes) = std::fs::read(&direct) {
            return serde_json::from_slice(&bytes)
                .map_err(|e| LotusError::State(format!("corrupt checkpoint: {e}")));
        }
    }
    let all = list(layout, key);
    let matches: Vec<&Checkpoint> = all.iter().filter(|c| c.id.starts_with(selector)).collect();
    match matches.len() {
        1 => Ok(matches[0].clone()),
        0 => Err(LotusError::NotFound(format!(
            "no checkpoint matching `{selector}`"
        ))),
        _ => Err(LotusError::Conflict(
            "checkpoint selector is ambiguous; use more characters".into(),
        )),
    }
}

/// Compare a checkpoint against current reality. Read-only.
pub fn compute_drift(layout: &Paths, cp: &Checkpoint) -> Vec<Drift> {
    let mut drift = Vec::new();
    let root = PathBuf::from(&cp.root);

    if !root.exists() {
        drift.push(Drift {
            kind: "root_missing".into(),
            expected: cp.root.clone(),
            found: "directory no longer exists".into(),
        });
        return drift;
    }

    // Manifest change?
    let manifest_path = root.join("lotus.toml");
    if let Ok(bytes) = std::fs::read(&manifest_path) {
        let current_hash = sha256_hex(&bytes);
        if current_hash != cp.manifest_hash {
            drift.push(Drift {
                kind: "manifest_changed".into(),
                expected: format!("hash {}", short(&cp.manifest_hash)),
                found: format!("hash {}", short(&current_hash)),
            });
        }
    } else {
        drift.push(Drift {
            kind: "manifest_missing".into(),
            expected: "lotus.toml present".into(),
            found: "lotus.toml not readable".into(),
        });
    }

    // Git drift?
    let ctx = gitctx::context(&root);
    if let Some(branch) = &cp.git_branch {
        match &ctx.branch {
            Some(current) if current != branch => drift.push(Drift {
                kind: "git_branch_changed".into(),
                expected: branch.clone(),
                found: current.clone(),
            }),
            None => drift.push(Drift {
                kind: "git_detached".into(),
                expected: branch.clone(),
                found: "(detached HEAD)".into(),
            }),
            _ => {}
        }
    }
    if let (Some(expected), Some(found)) = (&cp.git_commit, &ctx.commit) {
        if expected != found {
            drift.push(Drift {
                kind: "git_commit_changed".into(),
                expected: short(expected),
                found: short(found),
            });
        }
    }
    if ctx.dirty == Some(true) && cp.git_dirty == Some(false) {
        drift.push(Drift {
            kind: "workspace_dirty".into(),
            expected: "clean tree at checkpoint time".into(),
            found: "uncommitted changes present".into(),
        });
    }

    let _ = layout; // reserved for future runtime-state comparisons
    drift
}

fn short(hash: &str) -> String {
    hash.get(..12).unwrap_or(hash).to_string()
}
