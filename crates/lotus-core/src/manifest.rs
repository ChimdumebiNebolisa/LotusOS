use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{LotusError, Result};
use crate::util::sha256_hex;
use crate::MANIFEST_VERSION;

/// Raw `lotus.toml` schema. Unknown keys are rejected so typos fail loudly.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    version: u32,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    env: EnvSection,
    #[serde(default, rename = "process")]
    processes: Vec<RawProcess>,
    #[serde(default)]
    git: Option<GitSection>,
    #[serde(default)]
    paths: PathsSection,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct EnvSection {
    #[serde(default)]
    required: Vec<String>,
    #[serde(default)]
    env_files: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GitSection {
    #[serde(default)]
    required: bool,
    #[serde(default)]
    branch: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct PathsSection {
    #[serde(default)]
    checks: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProcess {
    name: String,
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    workdir: Option<String>,
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(default)]
    ports: Vec<u16>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    #[serde(default)]
    env_file: Option<String>,
    #[serde(default)]
    health: Option<HealthSpec>,
    #[serde(default)]
    restart: RestartSpec,
    #[serde(default)]
    shutdown: ShutdownSpec,
    #[serde(default)]
    version: Option<VersionSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealthSpec {
    /// TCP port to check (tcp or http kind).
    #[serde(default)]
    pub port: Option<u16>,
    /// HTTP path; requires `port`. Turns the port check into an HTTP GET.
    #[serde(default)]
    pub http_path: Option<String>,
    /// Expected HTTP status (default 200).
    #[serde(default)]
    pub expect_status: Option<u16>,
    /// Filesystem path relative to workspace root that must exist.
    #[serde(default)]
    pub path: Option<String>,
    /// Explicit command check (resolved like process commands).
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub command_args: Vec<String>,
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_startup_grace_ms")]
    pub startup_grace_ms: u64,
}

fn default_interval_ms() -> u64 {
    2000
}
fn default_timeout_ms() -> u64 {
    4000
}
fn default_startup_grace_ms() -> u64 {
    5000
}

impl Default for HealthSpec {
    fn default() -> Self {
        HealthSpec {
            port: None,
            http_path: None,
            expect_status: None,
            path: None,
            command: None,
            command_args: Vec::new(),
            interval_ms: default_interval_ms(),
            timeout_ms: default_timeout_ms(),
            startup_grace_ms: default_startup_grace_ms(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RestartPolicy {
    #[default]
    Never,
    OnFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestartSpec {
    #[serde(default)]
    pub policy: RestartPolicy,
    #[serde(default = "default_max_restarts")]
    pub max_restarts: u32,
    #[serde(default = "default_backoff_ms")]
    pub backoff_ms: u64,
}

fn default_max_restarts() -> u32 {
    3
}
fn default_backoff_ms() -> u64 {
    1000
}

impl Default for RestartSpec {
    fn default() -> Self {
        RestartSpec {
            policy: RestartPolicy::Never,
            max_restarts: default_max_restarts(),
            backoff_ms: default_backoff_ms(),
        }
    }
}

impl RestartSpec {
    pub fn on_failure(max_restarts: u32) -> Self {
        RestartSpec {
            policy: RestartPolicy::OnFailure,
            max_restarts,
            backoff_ms: default_backoff_ms(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShutdownSpec {
    #[serde(default = "default_grace_secs")]
    pub grace_secs: u64,
}

fn default_grace_secs() -> u64 {
    5
}

impl Default for ShutdownSpec {
    fn default() -> Self {
        ShutdownSpec {
            grace_secs: default_grace_secs(),
        }
    }
}

/// Optional runtime version expectation used by `doctor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionSpec {
    #[serde(default = "version_default_args")]
    pub args: Vec<String>,
    /// If set, the version output must contain this substring.
    #[serde(default)]
    pub contains: Option<String>,
}

fn version_default_args() -> Vec<String> {
    vec!["--version".to_string()]
}

/// A validated, resolved manifest.
#[derive(Debug, Clone)]
pub struct Manifest {
    pub schema_version: u32,
    pub name: String,
    pub description: Option<String>,
    /// Absolute directory of the manifest file.
    pub manifest_dir: PathBuf,
    /// Absolute workspace root (defaults to manifest dir).
    pub root: PathBuf,
    /// sha256 of the raw manifest bytes.
    pub hash: String,
    pub processes: Vec<Process>,
    pub env_required: Vec<String>,
    pub env_files: Vec<PathBuf>,
    pub git_required: bool,
    pub git_branch: Option<String>,
    pub path_checks: Vec<PathBuf>,
    /// Process names in dependency order.
    pub start_order: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Process {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    /// Absolute working directory (resolved from root).
    pub workdir: PathBuf,
    pub depends_on: Vec<String>,
    pub ports: Vec<u16>,
    pub env: BTreeMap<String, String>,
    /// Absolute env file path if declared.
    pub env_file: Option<PathBuf>,
    pub health: Option<HealthSpec>,
    pub restart: RestartSpec,
    pub shutdown: ShutdownSpec,
    pub version: Option<VersionSpec>,
}

impl Manifest {
    /// Parse and validate a `lotus.toml`. Returns the manifest plus its content hash.
    ///
    /// All validation problems are collected and reported together.
    pub fn load(path: &Path) -> Result<(Manifest, String)> {
        let raw_bytes = std::fs::read(path)
            .map_err(|e| LotusError::Manifest(format!("cannot read {}: {e}", path.display())))?;
        let hash = sha256_hex(&raw_bytes);
        let raw: RawManifest = toml::from_slice(&raw_bytes).map_err(|e| {
            LotusError::Manifest(format!("{} is not valid lotus.toml: {e}", path.display()))
        })?;

        let mut errors: Vec<String> = Vec::new();
        let manifest_dir = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        if raw.version != MANIFEST_VERSION {
            errors.push(format!(
                "unsupported manifest `version = {}`; this build understands version {MANIFEST_VERSION} only",
                raw.version
            ));
        }

        if raw.name.trim().is_empty() {
            errors.push("`name` must not be empty".into());
        }

        // ---- processes ----
        let mut names_seen = BTreeMap::new();
        for p in &raw.processes {
            if p.name.trim().is_empty() {
                errors.push("process with empty `name`".into());
                continue;
            }
            if names_seen.insert(p.name.clone(), ()).is_some() {
                errors.push(format!("duplicate process name `{}`", p.name));
            }
            if p.command.trim().is_empty() {
                errors.push(format!("process `{}` has empty `command`", p.name));
            }
            if p.workdir.as_deref().map(str::trim).is_some_and(str::is_empty) {
                errors.push(format!("process `{}` workdir must not be empty", p.name));
            }
            for dep in &p.depends_on {
                if dep == &p.name {
                    errors.push(format!(
                        "process `{}` depends on itself",
                        p.name
                    ));
                }
            }
            if p.ports.contains(&0) {
                errors.push(format!("process `{}` declares invalid port 0", p.name));
            }
        }

        // duplicate ports across processes in the same workspace
        let mut port_owner: BTreeMap<u16, &str> = BTreeMap::new();
        for p in &raw.processes {
            for port in &p.ports {
                match port_owner.get(port) {
                    Some(prev) => errors.push(format!(
                        "port {port} is declared by both `{prev}` and `{}`",
                        p.name
                    )),
                    None => {
                        port_owner.insert(*port, p.name.as_str());
                    }
                }
            }
        }

        // dependency references + cycle detection
        for p in &raw.processes {
            for dep in &p.depends_on {
                if !names_seen.contains_key(dep) {
                    errors.push(format!(
                        "process `{}` depends on unknown process `{dep}`",
                        p.name
                    ));
                }
            }
        }
        let start_order_result: std::result::Result<Vec<String>, Vec<String>> =
            topological_order(&raw.processes);
        if let Err(cycle) = &start_order_result {
            errors.push(format!(
                "cyclic process dependency detected involving: {}",
                cycle.join(", ")
            ));
        }
        let start_order = start_order_result.unwrap_or_default();

        // health specs
        for p in &raw.processes {
            if let Some(h) = &p.health {
                if h.http_path.is_some() && h.port.is_none() {
                    errors.push(format!(
                        "process `{}` health http_path requires `port`",
                        p.name
                    ));
                }
                let declared = [
                    h.port.is_some(),
                    h.path.is_some(),
                    h.command.is_some(),
                ]
                .iter()
                .filter(|b| **b)
                .count();
                if declared == 0 {
                    errors.push(format!(
                        "process `{}` health block declares no check (need one of port/path/command)",
                        p.name
                    ));
                }
                if h.interval_ms == 0 || h.timeout_ms == 0 {
                    errors.push(format!(
                        "process `{}` health interval_ms/timeout_ms must be > 0",
                        p.name
                    ));
                }
            }
        }

        if !errors.is_empty() {
            return Err(LotusError::Manifest(
                errors.into_iter().map(|e| format!("  - {e}")).collect::<Vec<_>>().join("\n"),
            ));
        }

        // Workspace root is the manifest's directory for V1.
        let root_dir = manifest_dir.clone();

        let processes = raw
            .processes
            .iter()
            .map(|p| {
                let workdir = match &p.workdir {
                    Some(rel) => root_dir.join(rel),
                    None => root_dir.clone(),
                };
                let env_file = p.env_file.as_ref().map(|f| root_dir.join(f));
                Process {
                    name: p.name.clone(),
                    command: p.command.clone(),
                    args: p.args.clone(),
                    workdir,
                    depends_on: p.depends_on.clone(),
                    ports: p.ports.clone(),
                    env: p.env.clone(),
                    env_file,
                    health: p.health.clone(),
                    restart: p.restart.clone(),
                    shutdown: p.shutdown,
                    version: p.version.clone(),
                }
            })
            .collect();

        let env_files: Vec<PathBuf> = raw
            .env
            .env_files
            .iter()
            .map(|f| root_dir.join(f))
            .collect();
        let path_checks: Vec<PathBuf> = raw
            .paths
            .checks
            .iter()
            .map(|c| root_dir.join(c))
            .collect();

        Ok((
            Manifest {
                schema_version: raw.version,
                name: raw.name,
                description: raw.description,
                manifest_dir,
                root: root_dir,
                hash: hash.clone(),
                processes,
                env_required: raw.env.required,
                env_files,
                git_required: raw.git.as_ref().map(|g| g.required).unwrap_or(false),
                git_branch: raw.git.and_then(|g| g.branch),
                path_checks,
                start_order,
            },
            hash,
        ))
    }

    pub fn find_process(&self, name: &str) -> Option<&Process> {
        self.processes.iter().find(|p| p.name == name)
    }
}

/// Kahn topological sort. Returns Err(list of processes involved in cycles).
fn topological_order(processes: &[RawProcess]) -> std::result::Result<Vec<String>, Vec<String>> {
    let names: BTreeSet<&str> = processes.iter().map(|p| p.name.as_str()).collect();
    let mut indegree: BTreeMap<&str, usize> = processes
        .iter()
        .map(|p| (p.name.as_str(), 0usize))
        .collect();
    let mut dependents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for p in processes {
        for dep in &p.depends_on {
            if names.contains(dep.as_str()) && dep != &p.name {
                indegree.entry(p.name.as_str()).and_modify(|d| *d += 1);
                dependents.entry(dep.as_str()).or_default().push(p.name.as_str());
            }
        }
    }

    let mut queue: std::collections::VecDeque<&str> = indegree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(n, _)| *n)
        .collect();
    let mut order = Vec::with_capacity(processes.len());
    while let Some(name) = queue.pop_front() {
        order.push(name.to_string());
        if let Some(deps) = dependents.get(name) {
            for d in deps.clone() {
                let e = indegree.get_mut(d).expect("known node");
                *e -= 1;
                if *e == 0 {
                    queue.push_back(d);
                }
            }
        }
    }

    if order.len() != processes.len() {
        let ordered: BTreeSet<&str> = order.iter().map(String::as_str).collect();
        let cyclic: Vec<String> = processes
            .iter()
            .filter(|p| !ordered.contains(p.name.as_str()))
            .map(|p| p.name.clone())
            .collect();
        return Err(cyclic);
    }
    Ok(order.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_manifest(dir: &Path, content: &str) -> PathBuf {
        let path = dir.join("lotus.toml");
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn parses_minimal_manifest() {
        let tmp = tempdir();
        let path = write_manifest(
            &tmp,
            r#"
version = 1
name = "demo"

[[process]]
name = "main"
command = "echo"
args = ["hi"]
"#,
        );
        let (m, hash) = Manifest::load(&path).unwrap();
        assert_eq!(m.schema_version, 1);
        assert_eq!(m.start_order, vec!["main"]);
        assert_eq!(hash.len(), 64);
        assert!(m.find_process("main").is_some());
    }

    #[test]
    fn rejects_unsupported_version() {
        let tmp = tempdir();
        let path = write_manifest(
            &tmp,
            r#"
version = 99
name = "demo"
[[process]]
name = "main"
command = "x"
"#,
        );
        let err = Manifest::load(&path).unwrap_err().to_string();
        assert!(err.contains("unsupported manifest"), "{err}");
    }

    #[test]
    fn rejects_missing_version_field() {
        let tmp = tempdir();
        let path = write_manifest(
            &tmp,
            r#"
name = "demo"
[[process]]
name = "main"
command = "x"
"#,
        );
        let err = Manifest::load(&path).unwrap_err().to_string();
        assert!(err.contains("not valid lotus.toml"), "{err}");
    }

    #[test]
    fn detects_duplicate_process_names() {
        let tmp = tempdir();
        let path = write_manifest(
            &tmp,
            r#"
version = 1
name = "demo"
[[process]]
name = "a"
command = "x"
[[process]]
name = "a"
command = "y"
"#,
        );
        let err = Manifest::load(&path).unwrap_err().to_string();
        assert!(err.contains("duplicate process name"), "{err}");
    }

    #[test]
    fn detects_cycles() {
        let tmp = tempdir();
        let path = write_manifest(
            &tmp,
            r#"
version = 1
name = "demo"
[[process]]
name = "a"
command = "x"
depends_on = ["b"]
[[process]]
name = "b"
command = "y"
depends_on = ["a"]
"#,
        );
        let err = Manifest::load(&path).unwrap_err().to_string();
        assert!(err.contains("cyclic"), "{err}");
    }

    #[test]
    fn detects_duplicate_ports_across_processes() {
        let tmp = tempdir();
        let path = write_manifest(
            &tmp,
            r#"
version = 1
name = "demo"
[[process]]
name = "a"
command = "x"
ports = [3000]
[[process]]
name = "b"
command = "y"
ports = [3000]
"#,
        );
        let err = Manifest::load(&path).unwrap_err().to_string();
        assert!(err.contains("both"), "{err}");
    }

    #[test]
    fn rejects_unknown_fields() {
        let tmp = tempdir();
        let path = write_manifest(
            &tmp,
            r#"
version = 1
name = "demo"
bogus_key = true
[[process]]
name = "a"
command = "x"
"#,
        );
        let err = Manifest::load(&path).unwrap_err().to_string();
        assert!(err.contains("not valid lotus.toml"), "{err}");
    }

    #[test]
    fn dependency_order_is_resolved() {
        let tmp = tempdir();
        let path = write_manifest(
            &tmp,
            r#"
version = 1
name = "demo"
[[process]]
name = "app"
command = "x"
depends_on = ["db", "cache"]
[[process]]
name = "db"
command = "x"
[[process]]
name = "cache"
command = "x"
depends_on = ["db"]
"#,
        );
        let (m, _) = Manifest::load(&path).unwrap();
        let pos = |n: &str| m.start_order.iter().position(|s| s == n).unwrap();
        assert!(pos("db") < pos("cache"));
        assert!(pos("cache") < pos("app"));
        assert!(pos("db") < pos("app"));
    }

    fn tempdir() -> PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let base = std::env::temp_dir().join(format!(
            "lotus-manifest-test-{}-{}-{}",
            std::process::id(),
            crate::util::now_ms(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&base).unwrap();
        base
    }
}
