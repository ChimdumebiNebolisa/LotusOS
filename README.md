# LotusOS

LotusOS is a custom Linux-based operating system for AI-native student and developer workflow.

It is not a from-scratch kernel, not a desktop app, and not a generic Linux theme. LotusOS is intended to become a full bootable Debian-based operating system image with opinionated defaults for coding, studying, research, files, projects, and local-first productivity.

Lotus Shell is the custom app/interface layer inside LotusOS. It is not the whole OS.

## Current Status

This repository now contains the initial Phase 0 and Phase 1 scaffold:

- Project direction docs.
- Debian `live-build` OS scaffold targeting Debian 13 `trixie`.
- KDE Plasma as the first desktop target.
- Package list drafts.
- Branding placeholder locations.
- Safe build, test, and clean script entrypoints.
- Reserved `shell/lotus-shell/` path for the future Tauri app.

No ISO has been built or verified yet. LotusOS is not installable yet. Lotus Shell is currently a planned first-class app, not an implemented app.

## MVP Target

The first successful LotusOS MVP must:

1. Build a bootable ISO.
2. Boot in QEMU.
3. Show LotusOS branding where feasible.
4. Land on a working KDE Plasma desktop session.
5. Include Lotus Shell preinstalled or clearly stubbed as a first-class app.
6. Include a basic default app set.
7. Include documented build and test commands.
8. Remain reproducible.

## Build Host

Use a Linux host or WSL2 Ubuntu environment. Debian `live-build` is not expected to run directly in PowerShell.

Install initial build dependencies inside WSL/Linux:

```bash
sudo apt update
sudo apt install live-build qemu-system-x86 xorriso isolinux syslinux-common squashfs-tools
```

Check the build environment without building an ISO:

```bash
bash os/scripts/build-iso.sh --check
```

Build the ISO:

```bash
bash os/scripts/build-iso.sh
```

Boot the ISO in QEMU:

```bash
bash os/scripts/test-qemu.sh artifacts/lotusos-amd64.iso
```

Clean generated build artifacts:

```bash
bash os/scripts/clean.sh --dry-run
bash os/scripts/clean.sh
```

## Project Docs

- Product vision: `docs/vision.md`
- Architecture: `docs/architecture.md`
- Roadmap: `docs/roadmap.md`
- Build notes: `docs/build.md`
- Decisions: `docs/decisions/`
- Verification notes: `docs/verification/`

