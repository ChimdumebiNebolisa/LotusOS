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

Status: verified.

- Configure live-build until an ISO is produced.
- Boot the ISO in QEMU.
- Verify KDE Plasma desktop session.
- Verify network tooling.
- Verify LotusOS branding where configured.
- Record results in `docs/verification/`.

## Phase 3: Lotus Shell Packaging

Status: verified through live-image packaging and prior visible-launch verification.

- Create or import the Tauri + React + TypeScript + Rust Lotus Shell app.
- Add Home, Projects, Notes, Files, AI Hub placeholder, and Settings placeholder screens.
- Package or copy Lotus Shell into the live image.
- Add a desktop launcher.
- Verify Lotus Shell opens in the live session.

## Phase 3C: Live Boot And Desktop Polish

Status: in progress.

- Keep the existing Debian `live-build` and Lotus Shell packaging path.
- Polish GRUB wording and reduce unattended boot delay.
- Make Lotus Shell autostart failures visible and logged without blocking KDE.
- Rebuild the ISO once.
- Verify the primary VirtualBox path with `VMSVGA` and 3D acceleration `on`.
- Document the `VMSVGA` plus 3D `off` `vmwgfx` caveat and any remaining desktop-state gaps.
- Do not mark complete until the supported `VMSVGA` plus 3D `on` path remains visibly stable through the acceptance window.
- Current blocker: VirtualBox graphics and session stability still degrade the live session after Lotus Shell becomes visible.

## Phase 4: Installer Integration

Status: partial verification.

- Add Calamares package and config.
- Add LotusOS installer branding.
- Verify installation into a VM.
- Document install limitations and recovery steps.
- Current state: the ISO builds with Calamares integrated and the supported VirtualBox GUI path reaches a visible KDE plus Lotus Shell state, but the installer launcher and open flow are not yet directly re-proven on-screen.

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
