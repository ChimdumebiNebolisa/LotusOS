# Phase 1 Verification: Scaffold

Date: 2026-05-08

## Files Changed

Phase 1 adds:

- Root docs and repo rules.
- `docs/` vision, architecture, roadmap, build, decision, and verification notes.
- `os/` live-build scaffold, package drafts, scripts, and branding placeholders.
- `shell/lotus-shell/` reservation docs.
- `.gitignore` entries for generated OS artifacts.

## Why Changed

The repository needed a clear OS track before any risky ISO build work. The scaffold defines LotusOS as a Debian-based operating system and reserves Lotus Shell as an app layer inside it.

## Commands To Run

```bash
git status --short
bash os/scripts/build-iso.sh --check
bash os/scripts/clean.sh --dry-run
```

## Commands Verified

From WSL Ubuntu:

```bash
bash -n os/scripts/build-iso.sh os/scripts/test-qemu.sh os/scripts/clean.sh os/live-build/auto/config os/live-build/auto/build os/live-build/auto/clean
bash os/scripts/clean.sh --dry-run
bash os/scripts/build-iso.sh --check
```

## Expected Result

- Git shows only intentional scaffold files.
- Build check reports host/dependency readiness without building an ISO.
- Clean dry-run lists only generated repo-local paths.

## Actual Result

- Bash syntax checks passed.
- Clean dry-run listed only `artifacts/` and generated `os/live-build/` paths.
- Build check reported missing WSL dependencies: `lb` and `xorriso`.
- No ISO build was attempted.

## Known Limitations

- The live-build config is an initial scaffold, not a verified ISO recipe.
- The Calamares installer is planned but not wired.
- Lotus Shell is reserved but not implemented.
- Branding assets are placeholders.
