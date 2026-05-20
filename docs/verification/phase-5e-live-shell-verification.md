# Phase 5E: Live ISO Shell Verification

Status: planned after Phase 5D ISO rebuild.

## Purpose

Boot the rebuilt `artifacts/lotusos-amd64.iso` and confirm the current Lotus Shell (Phases 5A through 8) appears in the live session.

## Supported VM Lane

- Backend: VirtualBox GUI
- Graphics: `VMSVGA`
- 3D acceleration: `on`
- RAM: `4096 MB`
- CPUs: `2`

## Checks

- GRUB reaches live boot.
- KDE live session appears.
- Lotus Shell autostarts.
- Home shows updated copy, session context, and launcher cards where installed.
- Settings shows read-only system overview.
- Projects, Notes, and Files show local resource cards when paths exist.

## Evidence To Capture

- Live-session screenshots.
- VM settings snapshot.
- Brief verification note in this file.
