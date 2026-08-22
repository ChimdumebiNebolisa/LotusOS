# Testing Guide

## Commands

```powershell
# fast unit tests (manifest, trust, logs, ports)
cargo test -p lotus-core --lib

# full adversarial integration suite (real processes; ~90 s on Windows)
cargo test -p lotus-core --test supervision

# compile + lint everything
cargo check --workspace
cargo clippy --workspace

# frontend type-check + build (Lotus Shell)
cd shell/lotus-shell && npm run build

# scripted end-to-end CLI lifecycle against the demo fixture
cargo build -p lotus-cli
powershell -File scripts\smoke-e2e.ps1
```

`scripts/smoke-e2e.ps1` isolates itself via `LOTUS_HOME`, so it never touches
real workspace state.

## What the suites actually cover

Unit (`--lib`, 12 tests): manifest parsing, unsupported versions, duplicate
names, cycles, duplicate ports, unknown-field rejection, dependency ordering,
trust round-trips, log rotation bounds.

Integration (`--test supervision`, 25+ tests) drives the real Engine with real
OS child processes:

| Adversarial scenario | Test |
|---|---|
| Healthy lifecycle OFF→STARTING→HEALTHY→STOPPING→OFF | `healthy_lifecycle_and_clean_stop` |
| Process exits immediately (clean) → DEGRADED | `immediate_clean_exit_degrades` |
| Crash + restart budget exhaustion → FAILED | `crash_restart_budget_then_failed` |
| Crash without policy stays crashed, no restarts | `crash_without_policy_stays_degraded_not_restarted` |
| Cyclic dependencies refused at add time | `dependency_order_and_cycle_refusal` |
| Missing executable → FAILED with detail | `missing_executable_marks_process_failed` |
| Invalid working directory → FAILED | `invalid_working_directory_marks_process_failed` |
| Command path containing spaces runs | `command_path_with_spaces_runs` |
| TCP health passes, then fails after listener dies | `tcp_health_passes_then_fails_after_listener_closes` |
| Startup grace prevents premature unhealthy | `startup_grace_prevents_premature_unhealthy` |
| HTTP health against a live endpoint | `http_health_check_against_real_endpoint` |
| Port conflict reported pre-flight, owner never killed | `preflight_port_conflict_reported_never_killed` |
| Two workspaces claiming the same port both diagnosed | `two_workspaces_same_port_both_diagnosed` |
| Orphan cleanup verifies identity before killing; skips mismatches | `orphan_cleanup_verifies_identity_before_killing` |
| Manifest change after trust blocks start until re-trust | `manifest_change_requires_retrust_before_start` |
| Untrusted workspace never starts | `untrusted_workspace_never_starts` |
| Doctor never prints env values | `doctor_never_prints_env_var_values` |
| Corrupt registry/trust/status files tolerated | `malformed_local_state_files_are_tolerated` |
| Checkpoint metadata + manifest drift report | `checkpoint_records_metadata_and_reports_manifest_drift` |
| Unknown checkpoint id refused | `restore_refuses_noop_when_checkpoint_selector_unknown` |
| Logs captured per stream with timestamps | `logs_are_captured_per_stream_with_timestamps` |
| STOPPING visible during shutdown window | `stopping_state_visible_during_stop` |
| Double start refused while running | `double_start_refused_while_running` |
| Fatal supervisor error leaves visible failed status | `fatal_startup_error_leaves_visible_failed_status` |

## Known gaps (do not assume coverage)

- Grandchild-process cleanup is exercised indirectly via tree-kill paths but
  has no dedicated cross-platform assertion.
- No automated test currently runs the Linux/macOS platform modules; they are
  cfg-gated out of this repo's verification (see platform-support.md).
- The Tauri desktop app is verified by compilation and by sharing every code
  path with the tested CLI/engine; its UI layer has no browser-automation
  tests.
- Ledger rotation and log rotation beyond two generations are not
  stress-tested.

## Fixtures

`fixtures/demo-workspace` for happy paths; `fixtures/invalid-manifests/*`
(cycle, bad version) demonstrate validation failures manually. See
`fixtures/README.md`.
