use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::util::{now_ms, sha256_hex};

/// Stable identity for a workspace: sha256 of its canonical root path
/// (lowercased on case-insensitive filesystems).
pub fn workspace_key(root: &Path) -> String {
    let canonical = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let text = canonical.to_string_lossy();
    #[cfg(target_os = "windows")]
    let text = text.to_lowercase();
    sha256_hex(text.as_bytes())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedEntry {
    pub name: String,
    pub root: String,
    pub manifest_hash: String,
    pub trusted_at_ms: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct TrustFile {
    version: u32,
    entries: BTreeMap<String, TrustedEntry>,
}

impl TrustFile {
    fn load(path: &Path) -> Self {
        match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|_| TrustFile {
                version: 1,
                entries: BTreeMap::new(),
            }),
            Err(_) => TrustFile {
                version: 1,
                entries: BTreeMap::new(),
            },
        }
    }

    fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(self)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

/// Read-only accessors used by the engine.
pub fn get_entry(trust_path: &Path, key: &str) -> Option<TrustedEntry> {
    TrustFile::load(trust_path).entries.get(key).cloned()
}

/// Grant (or re-grant) trust for a workspace at `root` with manifest hash `hash`.
pub fn grant(trust_path: &Path, key: &str, root: &Path, name: &str, hash: &str) -> Result<()> {
    let mut file = TrustFile::load(trust_path);
    file.entries.insert(
        key.to_string(),
        TrustedEntry {
            name: name.to_string(),
            root: root.to_string_lossy().to_string(),
            manifest_hash: hash.to_string(),
            trusted_at_ms: now_ms(),
        },
    );
    file.save(trust_path)
}

pub fn revoke(trust_path: &Path, key: &str) -> Result<bool> {
    let mut file = TrustFile::load(trust_path);
    let existed = file.entries.remove(key).is_some();
    if existed {
        file.save(trust_path)?;
    }
    Ok(existed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tempdir() -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "lotus-trust-test-{}-{}",
            std::process::id(),
            now_ms()
        ));
        std::fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn grant_get_revoke_roundtrip() {
        let dir = tempdir();
        let trust = dir.join("trust.json");
        let root = dir.join("ws");
        std::fs::create_dir_all(&root).unwrap();
        let key = workspace_key(&root);
        grant(&trust, &key, &root, "demo", "abc123").unwrap();
        let entry = get_entry(&trust, &key).expect("entry exists");
        assert_eq!(entry.manifest_hash, "abc123");
        // same directory => same key regardless of trailing separators
        assert_eq!(workspace_key(&root.join(".")), key);
        assert!(revoke(&trust, &key).unwrap());
        assert!(get_entry(&trust, &key).is_none());
    }
}
