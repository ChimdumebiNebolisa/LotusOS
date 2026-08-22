# Checkpoint Semantics

## What a checkpoint IS

A **deterministic metadata snapshot** of a workspace's operating context,
stored as JSON under `LOTUS_HOME/checkpoints/<workspace-key>/`:

```json
{
  "schema_version": 1,
  "id": "1787421872373-e935df82",
  "created_at_ms": 1787421872373,
  "workspace_key": "5e49130b...",
  "workspace_name": "demo",
  "root": "C:\\projects\\demo",
  "manifest_hash": "4c55be265ad2...",
  "git_branch": "main",
  "git_commit": "ba878593...",
  "git_dirty": false,
  "processes": ["api", "web"],
  "ports": [3000, 8080],
  "last_state": "healthy",
  "note": "before the refactor"
}
```

## What a checkpoint is NOT

It is not a memory dump. LotusOS does not pretend to serialize live process
state — that is impossible for arbitrary programs and dishonest to claim.
Process trees are restarted from their manifests; anything living only in
process memory is gone, exactly as it would be after a reboot.

## Restore behavior

`lotus restore <workspace> <checkpoint-id>` (or the desktop equivalent):

1. Loads the checkpoint and computes **drift** against current reality:
   - `root_missing` — workspace folder no longer exists
   - `manifest_changed` / `manifest_missing` — lotus.toml hash differs
   - `git_branch_changed`, `git_detached`, `git_commit_changed`
   - `workspace_dirty` — clean at checkpoint time, dirty now
2. Prints every drift item explicitly (none if identical).
3. Stops the workspace if running, then starts it fresh from the current
   manifest.

Restore therefore means: *reconstruct the declared lifecycle, while honestly
reporting what moved since the checkpoint*. It never silently rolls back files
— LotusOS does not touch your git state or revert code.

## Integrity notes

- Checkpoints reference content by SHA-256 hashes; drift detection compares
  current hashes, so tampering with either side is visible as drift rather
  than silent success.
- Corrupt or missing checkpoint files surface as explicit errors, not empty
  successes.
