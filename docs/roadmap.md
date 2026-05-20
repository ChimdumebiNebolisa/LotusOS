# LotusOS Roadmap

## Verified milestone state

- Phase 4 installer integration: `verified`
- Phase 5E live-shell verification: `verified`
- Phase 5F installed-shell verification: `manually verified after a Codex automation attempt was blocked by a live-session lock`
- Phase 6 identity polish: `implemented and locally verified`
- Phase 7 local app launcher surface: `implemented and locally verified`
- Phase 8 local resource hub: `implemented and locally verified`
- Phase 9 release hygiene: `completed in this preview closeout pass`

## Phase 0: Repo Audit And Project Direction

Status: completed.

## Phase 1: OS Build Scaffold

Status: completed.

## Phase 2: First Bootable Live ISO

Status: verified.

## Phase 3: Lotus Shell Packaging

Status: verified.

## Phase 3C: Live Boot And Desktop Polish

Status: verified with ongoing VirtualBox caveats.

- The supported `VMSVGA` + 3D `on` lane reaches KDE and Lotus Shell.
- VirtualBox graphics and session instability still appears in some timed runs and remains a documented limitation rather than a packaging blocker.

## Phase 4: Installer Integration

Status: verified.

- Calamares opens from the live session.
- Disposable VDI install flow is verified.
- Installed boot reaches SDDM, Plasma, and Lotus Shell.

## Phase 5A: Lotus Shell Home Polish

Status: implemented and locally verified.

## Phase 5B: Lotus Shell System Snapshot

Status: implemented and locally verified.

## Phase 5C: Tauri Packaging Cleanup

Status: verified locally on Windows.

## Phase 5D: Rebuilt ISO Packaging

Status: verified.

- `artifacts/lotusos-amd64.iso` was rebuilt with the current Lotus Shell binary.

## Phase 5E: Live ISO Shell Verification

Status: verified.

- Rebuilt ISO boots.
- KDE/live session reaches desktop.
- Lotus Shell autostarts.
- Updated Home and read-only Settings surfaces appear.

## Phase 5F: Installed System Shell Verification

Status: manually verified.

- Automated Calamares navigation was blocked when the live session locked.
- Manual follow-up verification completed the missing install, boot, greeter, login, Plasma, and Lotus Shell checks on the rebuilt ISO.
- This phase is intentionally recorded as manually verified, not fully automated.

## Phase 6: OS Identity Polish

Status: implemented and locally verified.

## Phase 7: Local App Launcher Surface

Status: implemented and locally verified.

## Phase 8: Local Resource Hub

Status: implemented and locally verified.

## Phase 9: Release Hygiene

Status: completed.

- README, roadmap, architecture, build notes, master plan, verification notes, and release notes are reconciled with the verified preview state.
- Curated README screenshots are copied from tracked local VM evidence only.

## Known limitations

- VirtualBox graphics and session instability remains a documented caveat.
- Hardware installation is not verified.
- LotusOS is not production-ready.
- ISO, VDI, and VM log artifacts remain out of git.

## Later

- Local search.
- Deeper repository and project-launch workflows.
- Study workspace expansion.
- PDF and document workflow depth.
- AI Hub integrations beyond the current placeholder/local-first stance.
