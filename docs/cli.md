# CLI Reference

Binary: `lotus` (`crates/lotus-cli`). Exit codes: `0` success, `1` operation
error, `2` usage error.

All commands accept a workspace **name** or a unique **key prefix** where a
`<workspace>` argument is shown.

## `lotus add <path> [--trust]`

Validate and register the workspace at `<path>`. Without `--trust`, prints the
full manifest review (every command, args, ports, required env names) and asks
`Trust this workspace...? [y/N]`. Default is No. See trust-model.md.

## `lotus list`

Table of registered workspaces: name, state, trust (plus MANIFEST CHANGED
drift marker), supervisor liveness, root.

## `lotus start <workspace>`

Refuses to run untrusted or changed manifests. Spawns a detached supervisor
(`lotus __supervise <key>`); returns after the first status heartbeat.

Windows scripting note: because the detached supervisor outlives the command,
shells that wait on inherited output handles (e.g. PowerShell pipelines) may
appear to hold; redirect output (`*> file`) or use `Start-Process` in scripts.
`scripts/smoke-e2e.ps1` demonstrates the safe pattern.

## `lotus stop <workspace>`

Requests stop via the control plane, waits until OFF bounded by the sum of the
manifest's shutdown grace periods + slack, then reports any orphan cleanup it
performed. Safe on already-stopped workspaces.

## `lotus restart <workspace>`

Stop (if running), then start.

## `lotus status <workspace>`

State, started time, per-process table (state/health/PID/restarts/detail),
port conflicts with remediation, last error if any.

## `lotus doctor <workspace>`

Read-only diagnostics: executable resolution per process, optional version
probes, env-var presence (**names only**), declared path existence/writability,
port conflicts, git expectations, trust state. Exit 1 when anything is not OK.
Statuses: OK / MISSING / INVALID / UNVERIFIED / CONFLICT.

## `lotus logs <workspace> [--process NAME] [--lines N]`

Most recent N lines (default 40) from captured process output, prefixed with
process/stream labels and timestamps. `--process` selects one process.

## `lotus events <workspace> [--limit N]`

Local lifecycle event ledger, newest page of an append-only JSONL history.

## `lotus checkpoint <workspace> [--note TEXT]`

Snapshot workspace metadata (manifest hash, git branch/commit/dirty, process
set, ports, last known state). See checkpoints.md.

## `lotus checkpoints <workspace>`

List checkpoints (id, time, git position, note).

## `lotus restore <workspace> <checkpoint-id>`

Prints a drift report against current reality (root present? manifest changed?
branch/commit moved? tree dirty?), then stops and restarts the workspace from
its manifest while reporting that drift. IDs may be abbreviated to a unique
prefix.

## `lotus trust <workspace>`

Show the current commands and re-grant trust (used after intentional manifest
changes).

## Hidden

- `lotus __supervise <key>` — internal supervisor entry point used by `start`.
