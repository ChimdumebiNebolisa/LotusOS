# LotusOS Master Plan

## Current Verified State

- Debian-based live ISO builds through the tracked `live-build` path in `os/scripts/build-iso.sh`.
- Live boot works, later phase evidence reaches the live graphical session, and KDE/live session is reached.
- Lotus Shell is packaged into the image and autostarts in the live session.
- Calamares opens from the live session.
- A disposable VDI install completes successfully.
- The installed VDI boots after ISO detach.
- The SDDM greeter appears.
- The Plasma desktop loads after installed login.
- Lotus Shell autostarts after installed login.
- Phase 5A is implemented and locally verified in the shell app: Home is polished, navigation works, and placeholder sections remain placeholder-only.
- Phase 5B is implemented and verified locally: read-only system snapshot, Home session context, Settings system overview.
- Phase 5C is verified locally on Windows: `icon.ico` added; `cargo check` and `npm run tauri build -- --no-bundle` pass. See `docs/verification/phase-5c-tauri-packaging.md`.
- Phase 6 identity polish, Phase 7 allowlisted local launchers, and Phase 8 local resource hub are implemented and verified locally in the shell app.

## Current Known Blockers

- VirtualBox graphics/session instability remains the main OS-level caveat. The supported `VMSVGA + 3D on` lane can reach Lotus Shell and still degrade to a black screen later in the observation window.
- Image-level VM verification gap: Phase 5D rebuilt `artifacts/lotusos-amd64.iso` with the current Lotus Shell binary, but Phases 5E and 5F still need live-session and installed-system proof.
- WSL ISO builds run as root must export the user `rustup` homes and toolchain on `PATH` (`CARGO_HOME`, `RUSTUP_HOME`, and `/home/<user>/.cargo/bin`); otherwise root sees no default toolchain or distro `cargo`/`rustc` below the `>= 1.85.0` guard.
- Phase 9 release hygiene is in progress: README, roadmap, architecture, build notes, and this master plan are being reconciled with the verified local shell work while ISO/VM evidence is still outstanding.

## Project Direction

LotusOS is:

- a custom Debian-based Linux ISO
- a KDE-based live/installable desktop image
- Lotus Shell as the home/workspace surface inside the OS
- local-first, simple, and intentionally polished

LotusOS is not:

- not a custom kernel project
- not a bootloader project
- not cloud-first
- not an AI API product yet
- not production hardware-certified

## Phase Roadmap

### Phase 5C: Tauri packaging cleanup

Goal: fix the missing icon/build prerequisite and make the full Tauri build path clean.

Scope:

- add the required Windows icon asset
- add any minimal Tauri packaging metadata needed for the existing app
- verify `cargo check` and no-bundle Tauri build behavior on the supported local host

Non-goals:

- no new user-facing Lotus Shell features
- no ISO rebuild
- no installer, GRUB, Calamares, or OS build-script changes

Files likely touched:

- `shell/lotus-shell/src-tauri/icons/`
- `shell/lotus-shell/src-tauri/tauri.conf.json`
- Tauri package metadata only if required for the existing build path

Acceptance criteria:

- `cargo check` no longer fails on the missing icon prerequisite
- `npm run tauri build -- --no-bundle` completes on the supported local host

Verification commands:

```bash
cd shell/lotus-shell
npm run build
cd src-tauri
cargo check
cd ..
npm run tauri build -- --no-bundle
```

### Phase 5D: Rebuild ISO with Phase 5A/5B Shell updates

Goal: package the updated Lotus Shell into the ISO.

Scope:

- rebuild using the existing `os/scripts/build-iso.sh` path only
- confirm the new shell binary is staged into the image
- confirm launcher and autostart files still point at the staged shell

Non-goals:

- no GRUB changes
- no installer changes
- no new shell features beyond already-merged Phase 5A/5B work

Acceptance criteria:

- ISO build succeeds
- updated Lotus Shell binary is present in the rebuilt image
- launcher and autostart files are still present in the image

Verification commands:

```bash
bash os/scripts/build-iso.sh --check
sudo bash os/scripts/build-iso.sh
```

Evidence expected:

- new build log
- image-content proof for Lotus Shell binary
- image-content proof for launcher/autostart files

### Phase 5E: Verify updated live ISO

Goal: boot the rebuilt ISO and verify the updated Lotus Shell appears in the live session.

Scope:

- live-session verification only
- confirm the updated Home and read-only Settings surfaces are visible
- use the current supported VirtualBox GUI lane as the decisive path

Non-goals:

- no install test yet
- no new shell features
- no backend or packaging changes during the verification pass

Acceptance criteria:

- GRUB reaches live boot
- KDE/live session appears
- Lotus Shell autostarts
- updated Home is visible
- updated read-only Settings surface is visible

Verification commands:

- boot the rebuilt ISO in the current supported VirtualBox GUI lane
- optional QEMU smoke check if useful, but VirtualBox GUI remains decisive

Evidence expected:

- live-session screenshots
- VM settings snapshot
- brief verification note

### Phase 5F: Verify updated installed system

Goal: install the rebuilt ISO to a disposable VDI and verify the updated Lotus Shell appears after installed login.

Scope:

- safe disposable install only
- confirm post-install boot, greeter, desktop, and Lotus Shell state

Non-goals:

- no installer rewrites
- no real-disk installs
- no new shell features

Acceptance criteria:

- disposable install completes
- installed VDI boots
- SDDM appears
- Plasma loads
- updated Lotus Shell autostarts after installed login

Evidence expected:

- installer completion screenshot
- installed boot screenshot
- installed desktop screenshot showing the updated shell

### Phase 6: OS identity polish

Goal: improve branding, icon, desktop naming, launcher naming, wallpaper, and first-run feel.

Scope:

- LotusOS-visible polish only
- align shell, launcher, and desktop-facing identity
- improve the first-run feel without inventing new core product capabilities

Non-goals:

- no kernel or bootloader engineering
- no deep theming system
- no new package manager or installer workflow

Acceptance criteria:

- shell, launcher, and desktop-facing names are consistent
- wallpaper/icon/identity feel intentional rather than generic
- live and installed surfaces no longer read as a generic scaffold where polish has been explicitly defined

### Phase 7: Local app launcher surface

Goal: add safe shortcuts to installed local apps and tools from Lotus Shell.

Scope:

- allowlisted launchers only for already-installed tools
- likely terminal, file manager, browser, editor, PDF, office, and installer where appropriate
- truthfully gate launcher visibility using session context

Non-goals:

- no arbitrary command execution
- no package manager UI
- no cloud integrations

Acceptance criteria:

- each launcher maps to a fixed known app
- unavailable launchers fail gracefully
- installer visibility is gated by truthful session context

### Phase 8: Local resource/project hub

Goal: add lightweight local project and resource cards without cloud sync or AI.

Scope:

- local-only project/resource surfaces
- narrow metadata or curated cards only
- preserve the lightweight shell direction

Non-goals:

- no notes database
- no sync
- no auth
- no AI API integrations

Acceptance criteria:

- shell exposes useful local project/resource entry points
- UI remains lightweight and honest about placeholders

### Phase 9: Release hygiene

Goal: clean docs, reproducible build instructions, known limitations, and the release checklist.

Scope:

- reconcile docs to the real verified state
- document known blockers and limitations
- create a preview release checklist that can actually be executed

Non-goals:

- no new app features
- no unsupported hardware claims

Acceptance criteria:

- docs consistently reflect verified vs planned work
- known blockers are explicit
- a preview release checklist exists and is actionable

## Branch Strategy

- Finish Phase 5B and this master-plan doc on the current local work branch, pushed to remote `phase-4-installer-integration`.
- After that, create a new `phase-5-shell-polish` branch for Phase 5C through 5F shell-and-packaging work.
- Merge to `main` only after:
- Phase 5C packaging cleanup is complete
- Phase 5D/5E prove the updated shell in the rebuilt live ISO
- Phase 5F proves the updated shell in the installed system
- Tag a preview release only after Phase 9 release hygiene is complete and the docs match the actual verified state.

## Verification Matrix

| Feature/Phase | Command or Manual Test | Expected Result | Evidence File | Status |
| --- | --- | --- | --- | --- |
| Phase 2 live ISO build | `bash os/scripts/build-iso.sh` and later live-boot verification flows | ISO builds and later verification reaches live boot | `docs/verification/phase-2-first-iso-build.md` plus later live-boot verification docs | Verified with doc drift |
| Phase 3 Lotus Shell packaging | Inspect image contents and boot live session | Lotus Shell binary is packaged and autostarts | `docs/verification/phase-3-lotus-shell-packaging.md` | Verified |
| Phase 3C live-session caveat | Supported VirtualBox GUI lane with timed observation | Lotus Shell can appear, but session stability remains incomplete | `docs/verification/phase-3c-live-boot-polish.md` | Partial |
| Phase 4 installer integration | Open Calamares, run disposable install, boot installed VDI | Installer opens, install completes, installed VDI boots into Plasma and Lotus Shell | `docs/verification/phase-4-installer-integration.md` | Verified |
| Phase 5A home polish | `cd shell/lotus-shell && npm run build` and local Vite checks | Home polish renders and placeholder navigation works | `docs/verification/phase-5a-lotus-shell-home.md` | Verified locally |
| Phase 5B system snapshot | `cd shell/lotus-shell && npm run build`; `cd src-tauri && cargo check` | Build passes; snapshot surfaces render locally | Local terminal output from `2026-05-20` | Verified locally |
| Phase 5C Tauri packaging cleanup | `cargo check`; `npm run tauri build -- --no-bundle` | Tauri packaging path is clean | `docs/verification/phase-5c-tauri-packaging.md` | Verified locally |
| Phase 5D rebuilt ISO packaging | `bash os/scripts/build-iso.sh --check`; WSL root build with user rustup on `PATH` | Rebuilt ISO contains updated shell | `docs/verification/phase-5d-iso-rebuild.md` | Verified (image content) |
| Phase 5E live ISO verification | Boot rebuilt ISO in supported VirtualBox GUI lane | Updated Home and Settings appear in live session | `docs/verification/phase-5e-live-shell-verification.md` | Planned |
| Phase 5F installed-system verification | Disposable install and installed login flow | Updated Lotus Shell appears after installed login | `docs/verification/phase-5f-installed-shell-verification.md` | Planned |

## Immediate Next Step

Run Phase 5E live-session and Phase 5F installed-system VM verification against `artifacts/lotusos-amd64.iso`, then finish Phase 9 release hygiene once evidence is recorded.
