# LotusOS

**LotusOS is a local-first developer workspace runtime.** It defines, starts,
supervises, diagnoses, stops, and restores the operating context of a software
project as **one coherent workspace**.

A development project is more than a folder. It has processes, ports,
environment requirements, logs, health conditions, Git state, and a lifecycle.
LotusOS lets you declare that context once in a versioned `lotus.toml` file and
then operate it as a unit — from a CLI (`lotus`) or the Lotus Shell desktop app.

- No AI, LLMs, or model providers required
- No cloud services, accounts, or external SaaS APIs
- No telemetry; all state lives under your own user profile
- V1 is Windows-first with an explicit platform adapter for Linux/macOS later

## Quick start

Prerequisites: [Rust](https://rustup.rs) 1.85+ (stable) on Windows 10/11.

```powershell
git clone https://github.com/ChimdumebiNebolisa/LotusOS.git
cd LotusOS
cargo build -p lotus-cli -p lotus-core
```

Try the demo workspace:

```powershell
# register + review + trust decision in one step
.\target\debug\lotus.exe add fixtures\demo-workspace --trust

.\target\debug\lotus.exe start demo      # OFF -> STARTING -> HEALTHY
.\target\debug\lotus.exe status demo     # processes, health, PIDs, restarts
.\target\debug\lotus.exe doctor demo     # environment diagnostics
.\target\debug\lotus.exe logs demo       # timestamped, per-stream logs
.\target\debug\lotus.exe stop demo       # graceful, then forced after grace
```

Or run the scripted end-to-end smoke test:

```powershell
powershell -File scripts\smoke-e2e.ps1
```

## The lifecycle

```text
OFF -> STARTING -> HEALTHY -> STOPPING -> OFF
                 |             |
                 +-> DEGRADED -+
                       |
                       v
                     FAILED
```

Every transition is deterministic and recorded in a local event ledger.
See [docs/lifecycle.md](docs/lifecycle.md).

## What's here

| Path | Contents |
|---|---|
| `crates/lotus-core` | The engine: manifest parsing, trust store, process supervisor, health checks, port diagnostics, doctor, logs, git context, checkpoints, event ledger |
| `crates/lotus-cli` | The `lotus` command-line interface |
| `shell/lotus-shell` | Lotus Shell desktop app (Tauri) built on the same engine |
| `fixtures/` | Demo and intentionally-invalid workspaces for testing |
| `scripts/smoke-e2e.ps1` | Scripted end-to-end verification |

## Documentation

- [`docs/architecture.md`](docs/architecture.md) — components and data flow
- [`docs/lotus-toml-spec.md`](docs/lotus-toml-spec.md) — manifest schema v1
- [`docs/trust-model.md`](docs/trust-model.md) — why and how trust is explicit
- [`docs/lifecycle.md`](docs/lifecycle.md) — states, transitions, semantics
- [`docs/cli.md`](docs/cli.md) — every command and exit codes
- [`docs/checkpoints.md`](docs/checkpoints.md) — what restore honestly does
- [`docs/platform-support.md`](docs/platform-support.md) — what works where
- [`docs/testing.md`](docs/testing.md) — how this repo is verified
- [`docs/migration.md`](docs/migration.md) — from the old OS-image product
- [`docs/archive/os-image-era/`](docs/archive/os-image-era/) — historical record of the retired Debian/ISO product

## Status

V0.2.0 — scaffolded and tested on Windows. Verified capabilities are recorded
in [`docs/testing.md`](docs/testing.md); anything not listed there has not been
verified. Do not assume Linux/macOS support until it appears in the platform
support matrix.

## License

MIT. See [LICENSE](LICENSE).
