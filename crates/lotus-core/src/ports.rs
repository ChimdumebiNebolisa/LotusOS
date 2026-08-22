//! Port ownership and collision diagnostics. LotusOS never kills another
//! process to free a port; it reports and advises.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::platform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConflict {
    pub port: u16,
    pub expected_owner: String,
    pub owner_pid: Option<u32>,
    pub owner_name: Option<String>,
    pub owned_by_workspace: bool,
    pub remediation: String,
}

/// Compare the ports a workspace expects to own against actual listeners.
///
/// `our_pids` are PIDs already belonging to this workspace (empty before start).
pub fn detect_conflicts(
    expected: &[(u16, String)],
    our_pids: &[u32],
) -> Result<Vec<PortConflict>> {
    let listeners = platform::list_tcp_listeners()?;
    let by_port: BTreeMap<u16, &platform::PortOwner> =
        listeners.iter().map(|l| (l.port, l)).collect();

    let mut conflicts = Vec::new();
    for (port, proc_name) in expected {
        if let Some(owner) = by_port.get(port) {
            let owned_by_workspace = owner
                .pid
                .map(|pid| our_pids.contains(&pid))
                .unwrap_or(false);
            if !owned_by_workspace {
                conflicts.push(PortConflict {
                    port: *port,
                    expected_owner: proc_name.clone(),
                    owner_pid: owner.pid,
                    owner_name: owner.name.clone(),
                    owned_by_workspace: false,
                    remediation: match (&owner.name, owner.pid) {
                        (Some(name), Some(pid)) => format!(
                            "stop `{name}` (pid {pid}) yourself or move this process off port {port}; LotusOS will not kill it"
                        ),
                        _ => format!(
                            "free port {port} manually or change the manifest; LotusOS will not kill the owning process"
                        ),
                    },
                });
            }
        }
    }
    Ok(conflicts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_listener_on_expected_port() {
        // Bind a real listener so the check exercises the live netstat path.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let _guard = listener;

        let expected = vec![(port, "web".to_string())];
        let conflicts = detect_conflicts(&expected, &[]).expect("listener query works");
        assert_eq!(conflicts.len(), 1, "port {port} should be reported");
        assert!(!conflicts[0].owned_by_workspace);
        assert!(conflicts[0].owner_pid.is_some());
        assert!(conflicts[0].remediation.contains("will not kill"));
    }

    #[test]
    fn no_conflict_when_port_free() {
        // Find a definitely-free port.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let conflicts = detect_conflicts(&[(port, "web".to_string())], &[]).unwrap();
        assert!(conflicts.is_empty());
    }
}
