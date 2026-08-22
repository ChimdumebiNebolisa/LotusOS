# `lotus.toml` Specification (schema version 1)

A workspace manifest is a TOML file named `lotus.toml` at the root of a project
folder. Unknown fields are rejected so typos fail loudly instead of being
ignored.

## Top level

```toml
version = 1              # REQUIRED, must be exactly 1
name = "my-app"          # REQUIRED, non-empty
description = "optional" # optional

[env]                    # optional section
required = ["DATABASE_URL"]   # variable NAMES only; values are never read for display
env_files = [".env"]          # loaded into every process environment at spawn

[git]                    # optional section
required = true               # doctor checks repository presence
branch  = "main"              # doctor checks current branch

[paths]                  # optional section
checks = ["data/", "config/app.toml"]   # existence + writability diagnostics

[[process]]              # REQUIRED: one or more process tables
...
```

## `[[process]]`

| Field | Type | Default | Notes |
|---|---|---|---|
| `name` | string | required | unique within the manifest |
| `command` | string | required | resolved via PATH; absolute paths allowed; `.bat`/`.cmd` run through `cmd /C` |
| `args` | `[string]` | `[]` | passed verbatim |
| `workdir` | path | workspace root | relative to the manifest directory; must exist at spawn |
| `depends_on` | `[string]` | `[]` | names of other processes; acyclic; controls start and reverse-stop order |
| `ports` | `[u16]` | `[]` | expected listeners used for conflict diagnostics |
| `env` | table | `{}` | inline variables applied after parent env and env file |

Use TOML literal strings (`'C:\path with spaces\tool.exe'`) for Windows paths so
backslashes stay verbatim.

### `[process.restart]`

```toml
policy = "on-failure"    # "never" (default) | "on-failure"
max_restarts = 3         # default 3 — total budget per supervised run
backoff_ms    = 1000     # default 1000; multiplied by attempt number
```

Only **failures** restart (non-zero exit or abnormal termination). A clean
exit is classified `exited` and degrades the workspace without restarting.

### `[process.shutdown]`

```toml
grace_secs = 5           # default 5
```

Stop sequence per process (reverse dependency order): best-effort graceful
termination first (Unix SIGTERM; Windows non-forced taskkill), wait up to the
grace period, then forced tree termination (SIGKILL to the process group;
Windows `taskkill /T /F`). See platform-support.md for caveats.

### `[process.health]`

All declared checks must pass. Checks are bounded by `timeout_ms`; failures do
not mark the process unhealthy until `startup_grace_ms` after spawn.

```toml
port         = 3000      # TCP connect check (or HTTP when http_path is set)
http_path    = "/healthz"  # requires port; plain-HTTP GET, expects expect_status
expect_status = 200
path         = "data/ready"   # must exist (relative to workspace root)
command      = "check-ready"  # explicit command probe; exit 0 = pass
command_args = []

interval_ms       = 2000   # default
timeout_ms        = 4000   # default; per-check bound
startup_grace_ms  = 5000   # default
```

Notes:
- HTTP checks are plain HTTP/1.0 GET over loopback only. TLS and redirects are
  out of scope in v1.
- Command probes resolve like process commands, run with the workspace root as
  cwd, and are killed at the timeout deadline.

### `[process.version]` (doctor-only)

```toml
args     = ["--version"]     # defaults to --version
contains = "v20"             # optional substring the output must include
```

## Validation rules

Collected and reported together on failure:

- `version` present and equal to 1
- non-empty `name`; at least one process
- unique process names; no self- or unknown-dependency references
- acyclic dependency graph (start order computed by topological sort)
- no duplicate `ports` across processes in one workspace; no port `0`
- health blocks declare at least one check; `http_path` requires `port`;
  interval/timeout > 0

Workdir existence, executable resolution, env-file presence are validated at
spawn time and reported as process failures with details.
