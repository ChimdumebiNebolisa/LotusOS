# LotusOS Architecture

## Product shape

LotusOS is a local-first developer workspace runtime. One core engine
(`lotus-core`) implements all workspace behavior; two frontends share it:

```text
                lotus.toml (workspace, on disk)
                        |
                        v
  +----------------- lotus-core -----------------+
  | manifest  trust  supervisor  health  ports   |
  | doctor    logs   gitctx      ledger registry |
  | checkpoint          platform adapter          |
  +-------------------+--------------+------------+
                      |              |
             crates/lotus-cli   shell/lotus-shell (Tauri)
             (`lotus` binary)     React UI over commands
```

There is exactly one implementation of every behavior. The CLI and the desktop
app cannot diverge because neither contains domain logic.

## Component responsibilities

| Module | Responsibility |
|---|---|
| `manifest` | Parse/validate `lotus.toml` v1; reject unknown fields; compute content hash; resolve dependency order (Kahn) and detect cycles |
| `trust` | Trust store keyed by canonical workspace root; records trusted manifest hash; detects material changes |
| `registry` | Registered workspaces (name, root, added-at) |
| `supervisor` | Dependency-ordered spawn, health polling, restart policy with budget/backoff, graceful-then-forced shutdown, crash classification, heartbeat status file, lock file, orphan cleanup |
| `health` | Bounded checks: TCP connect, plain-HTTP GET, path exists, command probe |
| `ports` | Listener discovery via platform adapter; conflict reports with remediation; never kills to free a port |
| `doctor` | Read-only environment diagnostics; distinguishes ok / missing / invalid / unverified / conflict |
| `logs` | Per-process `.out.log` / `.err.log` with timestamped lines and size-capped rotation |
| `gitctx` | Local-only git reads: branch, commit, dirty, ahead/behind from existing tracking refs |
| `ledger` | Append-only JSONL event history per workspace with rotation |
| `checkpoint` | Metadata snapshot (manifest hash, git position, process set, last state); drift computation |
| `platform` | The only OS-aware layer: executable resolution (PATH/PATHEXT), listener tables, PID identity tokens, tree termination |
| `engine` | Facade used by both frontends: add/list/start/stop/restart/status/doctor/logs/events/checkpoint/restore/trust |

## Control plane

Supervision is file-based so it works identically whether the supervisor runs as
a detached CLI child or an in-process thread in the desktop app:

```text
<LOTUS_HOME>/runtime/<key>/status.json    heartbeat written every ~250 ms tick
<LOTUS_HOME>/runtime/<key>/control.json   stop requests from any frontend
<LOTUS_HOME>/runtime/<key>/supervisor.lock  created exclusively; stale lock = dead supervisor
```

Readers treat a status older than 6 s as a **stale heartbeat** — that is how a
crashed LotusOS process is detected while workspace processes keep running.
Recovery kills recorded PIDs only after verifying each PID's platform identity
token (Windows creation FILETIME, Linux `/proc` starttime), never by PID alone.

## State ownership

| Data | Location | Writer |
|---|---|---|
| Manifest | `<workspace>/lotus.toml` | the developer |
| Registry, trust store | `LOTUS_HOME/*.json` | engine on explicit user action |
| Runtime heartbeats, control | `LOTUS_HOME/runtime/<key>/` | supervisor only |
| Logs | `LOTUS_HOME/logs/<key>/` | log-reader threads |
| Event ledger | `LOTUS_HOME/ledger/<key>/events.jsonl` | engine + supervisor |
| Checkpoints | `LOTUS_HOME/checkpoints/<key>/*.json` | engine |

`LOTUS_HOME` defaults to `%LOCALAPPDATA%\LotusOS` on Windows (XDG equivalent
elsewhere) and can be overridden for tests.

## Trust boundary

Executable definitions come from repositories, which may be hostile. The rule:

**No process is ever spawned from a workspace that has not been explicitly
trusted at its current manifest hash.**

Any byte change to `lotus.toml` invalidates trust until re-approved. Details:
[trust-model.md](trust-model.md).

## Deliberate non-goals

- No daemon manager, service installer, or system integration beyond user-level files
- No remote/network features of any kind
- No plugin/scripting DSL inside manifests
- No AI/LLM dependencies anywhere in the stack
