# Lifecycle Semantics

## Workspace states

```text
OFF -> STARTING -> HEALTHY -> STOPPING -> OFF
                 |             |
                 +-> DEGRADED -+
                       |
                       v
                     FAILED
```

| State | Meaning |
|---|---|
| `OFF` | Nothing supervised; no heartbeat (or terminal record only) |
| `STARTING` | Supervisor running; dependency-ordered spawning not finished, or processes running but health unknown/within grace |
| `HEALTHY` | All processes running and all declared health checks passing |
| `DEGRADED` | Running with at least one unhealthy process, an unexpected exit, or a crash |
| `FAILED` | At least one process failed terminally: spawn failure or exhausted restart budget |
| `STOPPING` | Stop requested; shutdown in progress |

Derivation is recomputed every supervision tick and written to the status
heartbeat. Transitions are recorded as `workspace_state_changed` events.

## Process states

`pending -> running -> {healthy | unhealthy}` plus
`restarting`, `exited`, `crashed`, `failed`, `stopping`, `stopped`.

Classification on unexpected termination:

- exit code 0 → `exited` (degrades the workspace; **not** restarted)
- non-zero or abnormal → `crashed`; restart policy applies if enabled

## Dependency ordering

Processes spawn in topological order of `depends_on`. A dependent starts only
after its dependencies reach `running`. Shutdown reverses the order.
Cycles are rejected at parse time.

## Restart policy

```toml
[process.restart]
policy = "on-failure"
max_restarts = 3      # total budget per supervised run
backoff_ms = 1000     # linear: attempt N waits backoff_ms * N
```

Exhausting the budget marks the process `failed` (workspace FAILED) and emits
`restart_exhausted`. The budget is per run, so a fresh `lotus start` resets it.

## Health evaluation

- First check fires immediately after spawn; then every `interval_ms`.
- Failures inside `startup_grace_ms` do not flip state — slow bootstraps are
  normal.
- After grace, any failing check marks the process `unhealthy`
  (workspace DEGRADED) and emits one `health_failed` event per transition;
  recovery emits `health_passed` and returns to healthy.
- Workspaces without declared health checks become HEALTHY once everything is
  running.

## Shutdown

1. Any frontend writes `control.json` stop request (or the supervisor receives
   SIGTERM-equivalent via the same file when LotusOS itself is asked to exit).
2. The supervisor writes a STOPPING heartbeat immediately, then terminates
   processes in reverse dependency order: graceful attempt, wait up to
   `grace_secs`, forced tree kill.
3. Final OFF status is written and the lock released.

Platform specifics (no SIGTERM on Windows consoles, tree-kill semantics) are in
[platform-support.md](platform-support.md).

## Crashed LotusOS recovery ("orphans")

If LotusOS dies while workspace processes survive:

- The stale heartbeat (>6 s old) marks the supervisor dead.
- The last recorded PIDs + identity tokens are used by cleanup:
  each PID is verified against its token before any kill; unverifiable PIDs
  are reported and skipped — never killed blind.
- Recovery runs automatically before a new start takes over the stale lock,
  and on demand from `stop`.

## Event ledger kinds

`workspace_added`, `trust_granted`, `start_requested`, `process_spawned`,
`spawn_failed`, `health_*`, `process_exited`, `crash_detected`,
`restart_scheduled`, `restart_exhausted`, `stop_requested`,
`process_stopped`, `workspace_state_changed`, `port_conflict_detected`,
`checkpoint_created`, `stop_completed`. Inspect with `lotus events <ws>`.
