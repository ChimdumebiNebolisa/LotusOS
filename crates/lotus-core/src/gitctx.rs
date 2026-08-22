//! Read-only local Git context. No network operations are ever performed;
//! ahead/behind is derived only from already-present remote-tracking refs.

use std::path::Path;

use serde::Serialize;

use crate::error::{LotusError, Result};

#[derive(Debug, Clone, Default, Serialize)]
pub struct GitContext {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub dirty: Option<bool>,
    /// From `status -sb` (local remote-tracking refs only; no fetch).
    pub ahead: Option<u32>,
    pub behind: Option<u32>,
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| LotusError::State(format!("git spawn failed: {e}")))?;
    if !output.status.success() {
        return Err(LotusError::State(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Gather local git state. Never touches the network.
pub fn context(root: &Path) -> GitContext {
    let mut ctx = GitContext::default();

    if git(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|v| v == "true")
        .unwrap_or(false)
    {
        ctx.is_repo = true;
    } else {
        return ctx;
    }

    if let Ok(branch) = git(root, &["symbolic-ref", "--short", "HEAD"]) {
        ctx.branch = Some(branch);
        ctx.commit = git(root, &["rev-parse", "HEAD"]).ok();
    } else {
        // detached HEAD
        ctx.branch = None;
        ctx.commit = git(root, &["rev-parse", "HEAD"]).ok();
    }

    if let Ok(status) = git(root, &["status", "--porcelain"]) {
        ctx.dirty = Some(!status.is_empty());
    }
    if let Ok(sb) = git(root, &["status", "-sb"]) {
        if let Some(first) = sb.lines().next() {
            for part in first.split_whitespace().skip(1) {
                if let Some(ahead) = part.strip_prefix("ahead ") {
                    ctx.ahead = ahead.parse().ok();
                }
                if let Some(behind) = part.strip_prefix("behind ") {
                    ctx.behind = behind.parse().ok();
                }
            }
        }
    }
    ctx
}
