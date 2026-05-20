# LotusOS

LotusOS is a custom Linux-based operating system for AI-native student and developer workflow.

It is not a from-scratch kernel, not a desktop app, and not a generic Linux theme. LotusOS is intended to become a full bootable Debian-based operating system image with opinionated defaults for coding, studying, research, files, projects, and local-first productivity.

Lotus Shell is the custom app/interface layer inside LotusOS. It is not the whole OS.

## Current Status

This repository now contains a verified live ISO build path and a packaged Lotus Shell scaffold:

- Project direction docs.
- Debian `live-build` OS scaffold targeting Debian 13 `trixie`.
- KDE Plasma as the first desktop target.
- A tracked Phase 2 GRUB boot repair path.
- Branding placeholder locations.
- Safe build, test, and clean script entrypoints.
- A Tauri + React + TypeScript + Rust Lotus Shell scaffold that is packaged into the live image and launched from the KDE session.

Current verification state:

- The ISO rebuild succeeds and produces `artifacts/lotusos-amd64.iso`.
- The rebuilt ISO GRUB menu now uses LotusOS wording and a `3` second timeout.
- VirtualBox verification is primarily supported with `VMSVGA` and 3D acceleration `on`.
- `VMSVGA` with 3D acceleration `off` still has the known `vmwgfx` instability and is not a packaging blocker.
- Phase 3C implementation exists and Lotus Shell does appear under `VMSVGA` with 3D acceleration `on`, but the session later degrades to a black screen, so stable desktop acceptance is still not re-proven.
- A follow-up live-session X11 anti-blanking and DPMS disable diagnostic did not prevent the later black-screen failure.
- The remaining blocker is VirtualBox graphics and session stability, not Lotus Shell packaging.
- Phase 4 installer integration is verified: Calamares opens, disposable VDI install completes, installed boot reaches SDDM and Plasma, and Lotus Shell autostarts after installed login. See `docs/verification/phase-4-installer-integration.md`.
- Phase 5A home polish, Phase 5B system snapshot, Phase 5C Tauri packaging cleanup, Phase 6 identity polish, Phase 7 local launchers, and Phase 8 local resource hub are implemented and verified locally in Lotus Shell.
- `artifacts/lotusos-amd64.iso` was rebuilt on `2026-05-20` with the current Lotus Shell (see `docs/verification/phase-5d-iso-rebuild.md`). Phase 5E/5F VM verification is still outstanding.

LotusOS still has a VirtualBox graphics/session stability caveat under the supported `VMSVGA` plus 3D `on` lane. Treat stable long-session desktop acceptance as not fully re-proven until fresh VM evidence is recorded for the rebuilt image.

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

Primary VirtualBox verification mode:

- Graphics controller: `VMSVGA`
- 3D acceleration: `on`
- RAM: `4096 MB`
- CPUs: `2`
- VRAM: `128 MB`

Known caveats:

- `VMSVGA` with 3D acceleration `off` can still hit the known `vmwgfx` unsupported-hypervisor instability and black-screen behavior.
- Under `VMSVGA` with 3D acceleration `on`, Lotus Shell can become visible and the session can still later degrade to a black screen.
- Disabling simple X11 blanking and DPMS in the live session did not fix that later degradation.

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
