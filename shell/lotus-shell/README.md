# Lotus Shell

Lotus Shell is the desktop (Tauri) frontend of the LotusOS workspace runtime.

It has no domain logic of its own: every command it exposes is a thin wrapper
over the shared engine in [`crates/lotus-core`](../../crates/lotus-core) — the
same engine the `lotus` CLI uses. The UI makes the workspace lifecycle the
central object: state, processes, health, ports, git context, doctor results,
logs, events, checkpoints, and start/stop/restart controls.

## Surfaces

- **Workspaces list** — registered workspaces with live state chips and trust/drift markers
- **Workspace detail** — process table, port conflicts, doctor findings, recent events, log tail, checkpoint create/restore-with-drift-report
- **Add flow** — inspect a `lotus.toml` before any trust decision, then register trusted or untrusted

## Development

```powershell
cd shell/lotus-shell
npm install
npm run dev          # frontend only (loopback Vite on :1420)
npm run tauri dev    # full desktop app
npm run build        # type-check + production bundle
```

Requires the Rust workspace to build (`cargo build` from the repo root).

## Notes

- Outside the Tauri runtime the app renders an honest preview notice; it never
  fakes local data.
- Generated Tauri schemas under `src-tauri/gen/` are local build artifacts and
  intentionally not committed.
