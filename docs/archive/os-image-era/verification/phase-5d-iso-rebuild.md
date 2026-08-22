# Phase 5D: Rebuilt ISO With Current Lotus Shell

Status: build and image-content verification complete; live and installed VM checks remain under Phases 5E and 5F.

## Build

- Command lane: WSL root with user `rustup` homes exported (`CARGO_HOME`, `RUSTUP_HOME`, and `/home/<user>/.cargo/bin` on `PATH`).
- Build log: `artifacts/vm-verification/phase5d-build-iso.log`
- ISO path: `artifacts/lotusos-amd64.iso`
- ISO size: `2465099776` bytes
- ISO timestamp: `2026-05-20 03:00:58 AM` (local)

## Image Content Verification

Verified from `filesystem.squashfs` in the rebuilt ISO:

- Evidence file: `artifacts/vm-verification/phase5d-iso-shell-paths.txt`
- `/opt/lotus-shell/lotus-shell` is present and executable.
- Lotus Shell launcher and autostart helper paths are present.
- Calamares installer launcher path remains present.

## Next

- Phase 5E: boot rebuilt ISO in the supported VirtualBox GUI lane and capture live-session evidence.
- Phase 5F: disposable install and installed-login verification.
