# Phase 5E: Live ISO Shell Verification

Status: verified.

## Purpose

Boot the rebuilt `artifacts/lotusos-amd64.iso` and confirm the current Lotus Shell live-session surfaces from Phases 5A through 5D appear without crossing the local-first guardrails.

## VM Settings

- Date: `2026-05-20`
- VM name: `LotusOS-Phase5E-Live-Test`
- Backend: `VirtualBox GUI`
- Graphics controller: `VMSVGA`
- 3D acceleration: `on`
- RAM: `4096 MB`
- CPUs: `2`
- VRAM: `128 MB`
- ISO path: `C:\Users\Chimdumebi\LotusOS\artifacts\lotusos-amd64.iso`

## Live Boot Result

- `20s`: GRUB auto-advanced into `Booting 'LotusOS Live'`
- `90s`: graphical wallpaper-only state appeared
- `~5m`: the session briefly fell into the same `vmwgfx` console/black-screen path seen in earlier VirtualBox caveats
- `~10m`: KDE recovered, Lotus Shell appeared, and the live session became usable

Classification:

- GRUB reached: `yes`
- GRUB auto-advanced: `yes`
- KDE/live session reached: `yes`
- Lotus Shell autostarted: `yes`
- Live ISO verification: `verified`
- Updated shell verified: `yes`

## Lotus Shell Verification

Home:

- Lotus Shell opened on `Home` by default.
- The updated Home dashboard headline `A calm first place to start.` appeared.
- The live-session runtime badges and local system context were populated.
- The Home cards matched the Phase 5A/5B direction: `Projects`, `Notes`, and `Files` destination cards; local launcher surfaces below; no cloud/auth/API prompts.

Settings:

- `Settings` opened successfully from the shell rail.
- The page showed the expected read-only `System Snapshot` surface.
- Runtime values were populated from the live session, including `LotusOS Preview`, `Debian GNU/Linux 13 (trixie)`, `Lotus`, `lotusos`, `Live session`, `Wayland`, and installer launcher presence.
- No writable preferences or speculative configuration controls appeared.

Projects, Notes, Files, and AI Hub:

- `Projects` showed local resource cards only (`/home/lotus`, `/home/lotus/Projects`, `/home/lotus/Code`) plus related local-tool launchers.
- `Notes` showed bounded local resource cards only (`/home/lotus/Notes`, `/home/lotus/Documents`) plus related local-tool launchers.
- `Files` showed bounded local resource cards only (`/home/lotus`, `/home/lotus/Desktop`, `/home/lotus/Documents`, `/home/lotus/Downloads`) plus related local-tool launchers.
- `AI Hub` remained a placeholder surface with the expected local-first guardrails (`No API keys bundled`, `No cloud account required`, `Future phase`).
- Repeated sidebar navigation across these sections did not break Lotus Shell.

Guardrail check:

- No cloud sync, auth, remote account setup, or API behavior appeared anywhere in the live-session shell verification.

## Caveats

- The supported VirtualBox lane still shows a delayed live-session recovery.
- The boot passed through an intermediate `vmwgfx` console and black-screen state before recovering to KDE and Lotus Shell.
- KDE `Welcome Center` opened on top of Lotus Shell after recovery and had to be dismissed before shell screenshots were captured.
- This verification was completed in the GUI frontend; the earlier headless caveat remains out of scope here.

## Evidence Files

- `artifacts/vm-verification/phase5e-live-20s.png`
- `artifacts/vm-verification/phase5e-live-90s.png`
- `artifacts/vm-verification/phase5e-live-desktop-lotus-shell.png`
- `artifacts/vm-verification/phase5e-live-home-dashboard.png`
- `artifacts/vm-verification/phase5e-live-settings-system-overview.png`
- Supplemental checks used during verification:
  - `artifacts/vm-verification/phase5e-live-10m-check.png`
  - `artifacts/vm-verification/phase5e-live-projects-check.png`
  - `artifacts/vm-verification/phase5e-live-notes-check.png`
  - `artifacts/vm-verification/phase5e-live-files-check.png`
  - `artifacts/vm-verification/phase5e-live-aihub-check.png`

## Result

Phase 5E is `verified` on the rebuilt ISO.

What is proven:

- The rebuilt ISO still boots through GRUB and reaches a usable KDE live session on the supported VirtualBox lane.
- Lotus Shell still autostarts in the live session.
- The updated Home dashboard is present in the live session.
- Settings exposes a populated read-only local system overview.
- Projects, Notes, Files, and AI Hub remain bounded local-first surfaces.

What remains uncertain:

- The VirtualBox GUI lane still recovers much more slowly than expected and still passes through a transient `vmwgfx` failure state before becoming usable.
