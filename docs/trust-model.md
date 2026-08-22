# Trust Model

## Threat

A workspace manifest is executable configuration that typically arrives by
cloning a repository. Cloning untrusted code is common; executing it should
never be silent or accidental. `lotus.toml` declares commands, arguments,
working directories, and environment files — all of which run with your user
privileges.

## Rule

**LotusOS never executes anything from a workspace until you explicitly trust
that workspace at its current content.**

Concretely:

1. `lotus add <path>` (or the desktop add flow) parses and validates the
   manifest, shows every command it would run, then asks for an explicit
   decision:
   - interactive prompt `[y/N]` (default No), or
   - non-interactive: pass `--trust` yourself after reviewing.
2. The decision records the SHA-256 of the exact manifest bytes in the local
   trust store (`LOTUS_HOME/trust.json`), keyed by a hash of the canonical
   workspace root.
3. Every start re-reads the manifest from disk. If its hash differs from the
   trusted hash, startup is refused:

```text
manifest changed since last trust decision (9f31c0aa -> 4c55be26); re-run `lotus trust`
```

4. `lotus trust <workspace>` (or "Review & trust" in the app) shows the current
   commands again and requires another explicit confirmation.

There is no wildcard, no global "trust everything", no bypass flag.

## What trust covers

- Any byte change to `lotus.toml` — including comments — invalidates trust.
  This is deliberate: cheap to re-approve, impossible to sneak a command edit
  past.
- Environment **files** declared in the manifest (`.env`) are part of runtime
  behavior but are not hashed; doctor reports whether they exist. Do not commit
  secret-bearing env files to shared repositories.

## Secret hygiene inside LotusOS

- `doctor` reports required environment variables by **name only**, never by
  value ("set (value not displayed)").
- Env values are passed into child process environments only; they are not
  logged, ledgered, checkpointed, or written anywhere by LotusOS itself.
- Workspace logs may contain whatever processes print; LotusOS adds timestamps
  and stream labels but cannot redact application output.
- Checkpoints store hashes and git positions, never environment values.

## Storage

| Item | Location |
|---|---|
| Trust entries | `%LOCALAPPDATA%\LotusOS\trust.json` (platform equivalent) |
| Registry | same directory, `workspaces.json` |

Both are user-local files; deleting them revokes all trust and forgets
registrations (workspace data itself is untouched).
