# Phase 10: VM Stability Verification

Status: Phase 10A build baseline recorded. Phase 10B VirtualBox baseline is `partial`. Phase 10D keepawake retest is `partial`.

## Purpose

Phase 10 is for reducing uncertainty around LotusOS VM stability before expanding OS behavior or making stronger install claims.

This phase should focus on repeatable evidence for the supported VirtualBox lane, especially the graphics/session instability seen during earlier live and installer verification. Do not treat Phase 10 as a broad feature phase.

## Baseline Repo State

- Branch: `phase-10-vm-stability`
- Latest commit: `TBD after Phase 10 preparation commit`
- Starting commit before Phase 10 preparation: `39fa5c4` (`Add LotusOS icon and wordmark logo assets.`)
- Latest tag: `v0.1.0-preview`
- Product state: Debian 13 `trixie` live/installable ISO customization with KDE Plasma, Calamares, and packaged Lotus Shell

## Supported VirtualBox Test Lane

- OS type: Debian 64-bit
- RAM: `4096 MB`
- CPUs: `2`
- VRAM: `128 MB`
- Graphics controller: `VMSVGA`
- 3D acceleration: `on`

## Test Checklist

- [ ] ISO rebuild completed or existing ISO freshness justified
- [ ] Live boot reaches GRUB and advances into `LotusOS Live`
- [ ] Live boot reaches a usable KDE Plasma session
- [ ] SDDM/session behavior observed where applicable
- [ ] Lotus Shell autostarts in the live session
- [ ] Calamares launches from the live session
- [ ] Calamares install target is confirmed as a disposable VDI before install
- [ ] Install completes to disposable VDI
- [ ] Post-install boot succeeds after ISO detach
- [ ] Post-install login reaches Plasma
- [ ] Lotus Shell launches after installed login

## Evidence Checklist

- [ ] Screenshots captured at meaningful boot/install milestones
- [ ] VM settings recorded
- [ ] ISO path and checksum recorded
- [ ] Calamares install target confirmation captured
- [ ] Result classified as `pass`, `partial`, or `blocker`
- [ ] Remaining uncertainty recorded without overclaiming

## Known Blockers

- Transient black screen during VirtualBox live-session recovery
- `vmwgfx` or graphics console instability
- Live-session lock during Calamares navigation
- Phase 5F installed-shell verification was manually verified but not automation-reproduced
- Hardware installation remains unverified

## Result

- Date: `2026-06-27` America/Chicago
- Tester: `ChimdumebiNebolisa`
- ISO path: `artifacts/lotusos-amd64.iso`
- ISO checksum: `SHA256 499fe4671d79a99dd7838ac1d9e5fb87eb2355eb158b2e215f92e73fde0b8719`
- VM name: `LotusOS-Phase10B-Baseline`
- VM settings: VirtualBox `7.2.8r173730` GUI frontend; OS type `Debian (64-bit)`; `4096 MB` RAM; `2` CPUs; `128 MB` VRAM; `VMSVGA`; 3D acceleration `on`; boot order DVD then disk; ISO attached at SATA port 1
- Disposable VDI path: `C:\Users\Chimdumebi\VirtualBox VMs\LotusOS-Phase10B-Baseline\LotusOS-Phase10B-Baseline.vdi`
- Live boot result: `partial`; GRUB/boot handoff showed `Booting 'LotusOS Live'` at `20s`, the live boot showed `vmwgfx` graphics-console errors at `90s`, KDE Plasma recovered by `5m`, then the session degraded to a black screen by `10m`
- Lotus Shell live-session result: `partial`; Lotus Shell autostarted and showed the Home dashboard at `5m`, but did not remain usable through the `10m` idle checkpoint
- Calamares launch result: not attempted because the live session degraded to black screen and then lock screen before safe installer navigation
- Install target confirmation: not reached
- Install result: not run
- Post-install boot result: not run
- Post-install Lotus Shell result: not run
- Classification: `partial`
- Evidence files: `artifacts/verification/iso-contents-20260628T004851Z.txt`; `artifacts/vm-verification/phase10b-baseline/phase10b-showvminfo-before.txt`; `artifacts/vm-verification/phase10b-baseline/phase10b-showvminfo-before-human.txt`; `artifacts/vm-verification/phase10b-baseline/phase10b-live-20s.png`; `artifacts/vm-verification/phase10b-baseline/phase10b-live-90s.png`; `artifacts/vm-verification/phase10b-baseline/phase10b-live-5m.png`; `artifacts/vm-verification/phase10b-baseline/phase10b-live-10m.png`; `artifacts/vm-verification/phase10b-baseline/phase10b-live-after-wake-attempt.png`; `artifacts/vm-verification/phase10b-baseline/phase10b-live-after-enter-unlock.png`; `artifacts/vm-verification/phase10b-baseline/phase10b-showvminfo-after-black-screen.txt`; `artifacts/vm-verification/phase10b-baseline/phase10b-showvminfo-after-poweroff.txt`
- Remaining uncertainty: Calamares launch, disposable VDI target confirmation, install completion, post-install boot, SDDM, installed Plasma, and installed Lotus Shell were not tested in this Phase 10B run because live-session stability failed before the installer step. Hardware installation remains unverified.

## Phase 10C Live Idle Stability Fix

Changed:

- Strengthened `os/live-build/config/includes.chroot/usr/local/bin/lotus-live-session-keepawake`.
- The helper now exits unless it detects a live session through `/run/live/medium`, `/lib/live/mount/medium`, or `boot=live` in `/proc/cmdline`.
- In live sessions, the helper disables KDE screen locking for the current live user by setting `Autolock=false`, `LockOnResume=false`, and `Timeout=0` in `kscreenlockerrc` through `kwriteconfig6` or `kwriteconfig5` when available.
- The existing `xset s off`, `xset -dpms`, and `xset s noblank` fallback remains for sessions where X11 `DISPLAY` is available.
- Updated the keepawake autostart desktop entry comment to reflect screen locking as well as blanking and DPMS.

Why this targets Phase 10B:

- Phase 10B reached KDE Plasma and Lotus Shell at `5m`, then degraded to a black screen by `10m`.
- The wake attempt reached a KDE lock screen, so disabling the live-session KDE locker is the smallest first target before changing graphics packages or VirtualBox settings.

What this does not claim:

- This does not claim to fix the earlier `vmwgfx` graphics-console errors.
- This does not claim live boot, Calamares launch, install completion, installed boot, SDDM, or installed Lotus Shell verification.
- This does not claim hardware support.

Installed-user impact:

- The autostart entry is still present system-wide, but the helper exits without changing settings when the system is not booted as a live session.
- No `/etc/skel` defaults were added, so this change is intended to affect the live verification session only.

Must retest:

- Rebuild the ISO so the updated helper is included.
- Re-run the supported VirtualBox lane through at least the Phase 10B `10m` idle checkpoint.
- If live session remains usable, continue to Calamares launch, disposable VDI target confirmation, install, post-install boot, SDDM, installed Plasma, and installed Lotus Shell checks.
- Keep `vmwgfx` errors separately tracked unless a later test directly proves they are resolved.

## Phase 10D Keepawake Retest

- Date: `2026-06-28` America/Chicago
- Tester: `ChimdumebiNebolisa`
- Rebuilt ISO path: `artifacts/lotusos-amd64.iso`
- Rebuilt ISO checksum: `SHA256 69eeb3a7bd6f139688a9b92c49428061394ad468a0ae087ff0499117c9199ae6`
- ISO contents evidence: `artifacts/verification/iso-contents-20260628T055625Z.txt`
- Keepawake inclusion: confirmed in the rebuilt ISO at `squashfs-root/usr/local/bin/lotus-live-session-keepawake` and `squashfs-root/etc/xdg/autostart/lotus-live-session-keepawake.desktop`
- Evidence directory: `artifacts/vm-verification/phase10d-keepawake-retest/`
- VM name: `LotusOS-Phase10D-Keepawake-Retest`
- VM settings: VirtualBox `7.2.8r173730` GUI frontend; OS type `Debian (64-bit)`; `4096 MB` RAM; `2` CPUs; `128 MB` VRAM; `VMSVGA`; 3D acceleration `on`; boot order DVD then disk; ISO attached at SATA port 1
- Disposable VDI path: `C:\Users\Chimdumebi\VirtualBox VMs\LotusOS-Phase10D-Keepawake-Retest\LotusOS-Phase10D-Keepawake-Retest.vdi`
- Live boot result: `partial`; `Booting 'LotusOS Live'` was visible at the early boot checkpoint, `vmwgfx` unsupported-hypervisor graphics-console errors still appeared during boot, KDE Plasma reached the graphical session, and Lotus Shell autostarted by the later live-session checkpoint
- Lotus Shell live-session result: `partial`; Lotus Shell was visible in the live session around the `8m` capture, but the display was black again at the `10m` and `12m` checkpoints
- Ten-minute black screen/lock result: not fixed; black-screen behavior recurred by the `10m` checkpoint. A KDE lock screen was not separately confirmed in this run because no wake/unlock attempt was made.
- `vmwgfx` result: persisted; the keepawake change did not resolve the graphics-console warning path
- Calamares launch result: not attempted because the live session was black at the `10m` and `12m` checkpoints and was not safe for installer navigation
- Install target confirmation: not reached
- Install result: not run
- Post-install boot result: not run
- Post-install Lotus Shell result: not run
- Classification: `partial`
- Evidence files: `artifacts/vm-verification/phase10d-keepawake-retest/vbox-settings.txt`; `artifacts/vm-verification/phase10d-keepawake-retest/vbox-settings-machinereadable.txt`; `artifacts/vm-verification/phase10d-keepawake-retest/phase10d-20s.png`; `artifacts/vm-verification/phase10d-keepawake-retest/phase10d-90s.png`; `artifacts/vm-verification/phase10d-keepawake-retest/phase10d-midway-before-5m.png`; `artifacts/vm-verification/phase10d-keepawake-retest/phase10d-5m.png`; `artifacts/vm-verification/phase10d-keepawake-retest/phase10d-8m.png`; `artifacts/vm-verification/phase10d-keepawake-retest/phase10d-10m.png`; `artifacts/vm-verification/phase10d-keepawake-retest/phase10d-12m.png`; `artifacts/vm-verification/phase10d-keepawake-retest/VBox.log`; `artifacts/vm-verification/phase10d-keepawake-retest/vbox-final-before-poweroff.txt`; `artifacts/vm-verification/phase10d-keepawake-retest/vbox-final-after-poweroff.txt`
- Remaining uncertainty: Calamares launch, disposable VDI target confirmation, install completion, post-install boot, SDDM, installed Plasma, installed Lotus Shell, and hardware installation remain unverified in Phase 10D.
