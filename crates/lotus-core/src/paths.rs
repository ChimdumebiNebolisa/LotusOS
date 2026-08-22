use std::path::PathBuf;

/// Root directory for all LotusOS local state.
///
/// Resolution order:
/// 1. `LOTUS_HOME` environment variable (used by tests to isolate state)
/// 2. Windows: `%LOCALAPPDATA%\LotusOS`
/// 3. macOS:   `~/Library/Application Support/LotusOS`
/// 4. Other:   `$XDG_DATA_HOME/LotusOS` or `~/.local/share/LotusOS`
pub fn data_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("LOTUS_HOME").filter(|v| !v.is_empty()) {
        return PathBuf::from(home);
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(local) = std::env::var_os("LOCALAPPDATA").filter(|v| !v.is_empty()) {
            return PathBuf::from(local).join("LotusOS");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME").filter(|v| !v.is_empty()) {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("LotusOS");
        }
    }

    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME").filter(|v| !v.is_empty()) {
        return PathBuf::from(xdg).join("LotusOS");
    }
    if let Some(home) = std::env::var_os("HOME").filter(|v| !v.is_empty()) {
        return PathBuf::from(home).join(".local/share/LotusOS");
    }
    PathBuf::from(".")
}

pub fn layout(base: &std::path::Path) -> Paths {
    Paths {
        base: base.to_path_buf(),
        registry: base.join("workspaces.json"),
        trust: base.join("trust.json"),
        runtime: base.join("runtime"),
        logs: base.join("logs"),
        ledger: base.join("ledger"),
        checkpoints: base.join("checkpoints"),
    }
}

#[derive(Clone, Debug)]
pub struct Paths {
    pub base: PathBuf,
    pub registry: PathBuf,
    pub trust: PathBuf,
    pub runtime: PathBuf,
    pub logs: PathBuf,
    pub ledger: PathBuf,
    pub checkpoints: PathBuf,
}

impl Paths {
    /// Per-workspace subdirectory under a given top-level folder.
    pub fn for_workspace(&self, root: &std::path::Path, key: &str, file: Option<&str>) -> PathBuf {
        let dir = root.join(key);
        match file {
            Some(f) => dir.join(f),
            None => dir,
        }
    }

    pub fn status_file(&self, key: &str) -> PathBuf {
        self.runtime.join(key).join("status.json")
    }

    pub fn control_file(&self, key: &str) -> PathBuf {
        self.runtime.join(key).join("control.json")
    }

    pub fn lock_file(&self, key: &str) -> PathBuf {
        self.runtime.join(key).join("supervisor.lock")
    }

    pub fn ensure(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.base)?;
        std::fs::create_dir_all(&self.runtime)?;
        std::fs::create_dir_all(&self.logs)?;
        std::fs::create_dir_all(&self.ledger)?;
        std::fs::create_dir_all(&self.checkpoints)?;
        Ok(())
    }
}
