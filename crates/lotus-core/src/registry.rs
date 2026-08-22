use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::util::now_ms;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredWorkspace {
    pub name: String,
    pub root: String,
    pub added_at_ms: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct RegistryFile {
    version: u32,
    workspaces: BTreeMap<String, RegisteredWorkspace>,
}

impl RegistryFile {
    fn load(path: &Path) -> Self {
        match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|_| RegistryFile {
                version: 1,
                workspaces: BTreeMap::new(),
            }),
            Err(_) => RegistryFile {
                version: 1,
                workspaces: BTreeMap::new(),
            },
        }
    }

    fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }
}

pub fn add(registry_path: &Path, key: &str, name: &str, root: &Path) -> Result<()> {
    let mut file = RegistryFile::load(registry_path);
    file.workspaces.insert(
        key.to_string(),
        RegisteredWorkspace {
            name: name.to_string(),
            root: root.to_string_lossy().to_string(),
            added_at_ms: now_ms(),
        },
    );
    file.save(registry_path)
}

pub fn remove(registry_path: &Path, key: &str) -> Result<bool> {
    let mut file = RegistryFile::load(registry_path);
    let existed = file.workspaces.remove(key).is_some();
    if existed {
        file.save(registry_path)?;
    }
    Ok(existed)
}

pub fn get(registry_path: &Path, key: &str) -> Option<RegisteredWorkspace> {
    RegistryFile::load(registry_path).workspaces.get(key).cloned()
}

pub fn all(registry_path: &Path) -> BTreeMap<String, RegisteredWorkspace> {
    RegistryFile::load(registry_path).workspaces
}

pub fn find_by_name_or_key(
    registry_path: &Path,
    selector: &str,
) -> Option<(String, RegisteredWorkspace)> {
    let all = all(registry_path);
    if let Some((key, ws)) = all.iter().find(|(_, ws)| ws.name == selector) {
        return Some((key.clone(), ws.clone()));
    }
    // unique prefix match on the workspace key
    let matches: Vec<(String, RegisteredWorkspace)> = all
        .into_iter()
        .filter(|(k, _)| k.starts_with(selector))
        .collect();
    if matches.len() == 1 {
        Some(matches.into_iter().next().expect("len checked"))
    } else {
        None
    }
}
