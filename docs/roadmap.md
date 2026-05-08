# LotusOS Roadmap

## Phase 0: Repo Audit And Project Direction

Status: scaffolded.

- Inspect repository state.
- Confirm whether existing code is OS build system, Lotus Shell, or unrelated.
- Define product niche and MVP boundary.
- Add vision, architecture, roadmap, build, decisions, and verification docs.

## Phase 1: OS Build Scaffold

Status: scaffolded, not verified as a successful build.

- Add `os/` structure.
- Add Debian `live-build` scaffold.
- Add package list drafts.
- Add branding placeholder paths.
- Add safe build, test, and clean scripts.
- Document WSL/Linux build host requirements.

## Phase 2: First Bootable Live ISO

Status: planned.

- Configure live-build until an ISO is produced.
- Boot the ISO in QEMU.
- Verify KDE Plasma desktop session.
- Verify network tooling.
- Verify LotusOS branding where configured.
- Record results in `docs/verification/`.

## Phase 3: Lotus Shell Packaging

Status: planned.

- Create or import the Tauri + React + TypeScript + Rust Lotus Shell app.
- Add Home, Projects, Notes, Files, AI Hub placeholder, and Settings placeholder screens.
- Package or copy Lotus Shell into the live image.
- Add a desktop launcher.
- Verify Lotus Shell opens in the live session.

## Phase 4: Installer Integration

Status: planned.

- Add Calamares package and config.
- Add LotusOS installer branding.
- Verify installation into a VM.
- Document install limitations and recovery steps.

## Phase 5: Niche Features

Status: planned after boot and install reliability.

- Project dashboard.
- Notes and resource hub.
- Local file/project organization.
- Local search.
- Git/repository launcher.
- Study workspace.
- PDF/document workflow.
- AI Hub integrations.

