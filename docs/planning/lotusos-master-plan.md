# LotusOS Master Plan

## Current Verified State

- Debian-based live ISO builds through the tracked `live-build` path in `os/scripts/build-iso.sh`
- Live boot works and KDE/live session is reached
- Lotus Shell is packaged into the image and autostarts in the live session
- Calamares opens from the live session
- A disposable VDI install completes successfully
- The installed VDI boots after ISO detach
- The SDDM greeter appears
- The Plasma desktop loads after installed login
- Lotus Shell autostarts after installed login
- Phase 5E is verified on the rebuilt ISO: live boot reaches KDE, Lotus Shell autostarts, the updated Home dashboard appears, and Settings shows the read-only system overview
- Phase 5F is manually verified on the rebuilt ISO: after a blocked automated Calamares attempt, the user manually verified install completion, installed boot, SDDM, Plasma, and the updated Lotus Shell surfaces
- Phase 5A is implemented and locally verified in the shell app: Home is polished, navigation works, and placeholder sections remain placeholder-only
- Phase 5B is implemented and verified locally: read-only system snapshot, Home session context, Settings system overview
- Phase 5C is verified locally on Windows: `icon.ico` added; `cargo check` and `npm run tauri build -- --no-bundle` pass
- Phase 6 identity polish, Phase 7 allowlisted local launchers, and Phase 8 local resource hub are implemented and verified locally in the shell app
- Phase 9 release hygiene is complete in this closeout pass

## Current Known Blockers

- VirtualBox graphics/session instability remains the main OS-level caveat. The supported `VMSVGA + 3D on` lane can reach Lotus Shell and still degrade to a black screen later in the observation window
- WSL ISO builds run as root must export the user `rustup` homes and toolchain on `PATH` (`CARGO_HOME`, `RUSTUP_HOME`, and `/home/<user>/.cargo/bin`); otherwise root sees no default toolchain or distro `cargo` and `rustc` below the `>= 1.85.0` guard
- Hardware installation support is still unverified
- No production release guarantee exists for this preview milestone

## Project Direction

LotusOS is:

- a custom Debian-based Linux ISO
- a KDE-based live/installable desktop image
- Lotus Shell as the home/workspace surface inside the OS
- local-first, simple, and intentionally polished

LotusOS is not:

- a custom kernel project
- a bootloader project
- cloud-first
- an AI API product yet
- production hardware-certified

## Phase Roadmap

### Completed through preview closeout

- Phase 4: verified installer integration
- Phase 5E: verified live-session shell surfaces on the rebuilt ISO
- Phase 5F: manually verified installed-session shell surfaces after the blocked automation attempt
- Phase 6: implemented and locally verified identity polish
- Phase 7: implemented and locally verified local app launchers
- Phase 8: implemented and locally verified local resource hub
- Phase 9: completed documentation and release closeout

### Remaining work after the preview milestone

- Reduce VirtualBox graphics instability
- Re-verify broader VM lanes and real hardware before making stronger support claims
- Expand Lotus Shell only after the current local-first surfaces remain stable through longer OS-level validation

## Branch Strategy

- The preview closeout has been merged to `main` and tagged as `v0.1.0-preview`
- Phase 10 work should branch from the current clean `main`
- Keep VM stability changes small and evidence-led until the current blocker is reproduced clearly

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
| Phase 5E live ISO verification | Boot rebuilt ISO in supported VirtualBox GUI lane | Updated Home and Settings appear in live session | `docs/verification/phase-5e-live-shell-verification.md` | Verified |
| Phase 5F installed-system verification | Disposable install and installed login flow | Updated Lotus Shell appears after installed login | `docs/verification/phase-5f-installed-shell-verification.md` | Manually verified |

## Immediate Next Step

Begin Phase 10 VM stability preparation and verification from the current clean `main` baseline.
