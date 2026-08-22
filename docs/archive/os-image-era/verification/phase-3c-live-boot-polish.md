# Phase 3C: Live Boot And Desktop Polish Verification

Status: partial verification.

## Classification

- Build result: `success`
- GRUB polish result: `success`
- VirtualBox desktop-state result: `unstable after anti-blanking diagnostic`

Phase 3C is still not complete. A single follow-up diagnostic disabled simple X11 blanking and DPMS in the live session, rebuilt the ISO once, and re-ran the supported VirtualBox lane (`VMSVGA` with 3D `on`). The result still failed acceptance: the session was wallpaper-only at `5m`, Lotus Shell was visible at `10m`, and the VM degraded to a black screen at `15m` and `20m`. Because the black screen persists after disabling simple blanking and DPMS, the remaining issue is best documented as VirtualBox graphics instability rather than a Phase 3C completion.

## Files Changed

- `os/scripts/build-iso.sh`
- `os/live-build/config/includes.chroot/usr/local/bin/lotus-shell-launch`
- `os/live-build/config/includes.chroot/usr/local/bin/lotus-live-session-keepawake`
- `os/live-build/config/includes.chroot/etc/xdg/autostart/lotus-shell.desktop`
- `os/live-build/config/includes.chroot/etc/xdg/autostart/lotus-live-session-keepawake.desktop`
- `README.md`
- `docs/build.md`
- `docs/architecture.md`
- `docs/roadmap.md`
- `docs/verification/phase-3c-live-boot-polish.md`

## ISO

- Path: `artifacts/lotusos-amd64.iso`
- Size: `2446880768` bytes
- Timestamp: `2026-05-17 07:15:12 PM -0500`
- Build log: `artifacts/vm-verification/phase3c-keepawake-build-iso.log`

## Verified Build Changes

Verified from the rebuilt ISO:

- GRUB timeout is `3` seconds.
- GRUB default entry label is `LotusOS Live`.
- GRUB safe entry label is `LotusOS Live (safe graphics)`.
- Kernel-specific labels use LotusOS wording.

Verified from the live-build config and chroot include copy:

- Lotus Shell autostart still runs through `/usr/local/bin/lotus-shell-launch`.
- The Lotus Shell wrapper behavior is unchanged by this diagnostic.
- A separate live-session autostart entry now runs `/bin/sh /usr/local/bin/lotus-live-session-keepawake`.
- The keepawake helper applies:
  - `xset s off`
  - `xset -dpms`
  - `xset s noblank`

## Primary VirtualBox Mode

- OS type: Debian 64-bit
- RAM: `4096 MB`
- CPUs: `2`
- VRAM: `128 MB`
- Graphics controller: `VMSVGA`
- 3D acceleration: `on`

## Evidence

- `artifacts/vm-verification/phase3c-build-iso.log`
- `artifacts/vm-verification/phase3c-controlled-vmsvga-3d-5m.png`
- `artifacts/vm-verification/phase3c-controlled-vmsvga-3d-10m.png`
- `artifacts/vm-verification/phase3c-controlled-vmsvga-3d-15m.png`
- `artifacts/vm-verification/phase3c-controlled-vmsvga-3d-20m.png`
- `artifacts/vm-verification/phase3c-controlled-vmsvga-3d-showvminfo-before.txt`
- `artifacts/vm-verification/phase3c-controlled-vmsvga-3d-showvminfo-after.txt`
- `artifacts/vm-verification/phase3c-keepawake-build-iso.log`
- `artifacts/vm-verification/phase3c-keepawake-vmsvga-3d-5m.png`
- `artifacts/vm-verification/phase3c-keepawake-vmsvga-3d-10m.png`
- `artifacts/vm-verification/phase3c-keepawake-vmsvga-3d-15m.png`
- `artifacts/vm-verification/phase3c-keepawake-vmsvga-3d-20m.png`
- `artifacts/vm-verification/phase3c-keepawake-vmsvga-3d-showvminfo-before.txt`
- `artifacts/vm-verification/phase3c-keepawake-vmsvga-3d-showvminfo-after.txt`

## VirtualBox Result

- Reaches GRUB: yes
- GRUB auto-advances with LotusOS wording: yes
- Starts live boot: yes
- Reaches graphical Debian/KDE state: yes
- Reaches a clearly visible KDE desktop through `20m`: no
- Lotus Shell visibly launched: yes, at `10m` in the keepawake diagnostic run
- Simple X11 anti-blanking result: insufficient

Evidence notes:

- Prior controlled pass:
  - `phase3c-controlled-vmsvga-3d-5m.png`: Lotus Shell is visible.
  - `phase3c-controlled-vmsvga-3d-10m.png`, `15m`, and `20m`: the session is black.
- Keepawake diagnostic pass:
  - `phase3c-keepawake-vmsvga-3d-5m.png`: only the Debian 13 wallpaper is visible.
  - `phase3c-keepawake-vmsvga-3d-10m.png`: Lotus Shell is visible with the desktop/session still present.
  - `phase3c-keepawake-vmsvga-3d-15m.png` and `phase3c-keepawake-vmsvga-3d-20m.png`: the session is black.

## Known Caveats

- VirtualBox `VMSVGA` with 3D acceleration `off` still reproduces the known `vmwgfx` unsupported-hypervisor instability and remains a documented caveat rather than a packaging blocker.
- Under the supported `VMSVGA` plus 3D `on` mode, disabling simple X11 blanking and DPMS does not keep the session visible through `20m`.

## Conclusion

Phase 3C remains partial verification and should not be marked complete or committed as complete. The implementation exists, the ISO rebuild succeeded, and GRUB polish is verified. The single minimal diagnostic changed the timing of what was visible, but it did not eliminate the black-screen failure: the supported VirtualBox lane still degrades to black by `15m` and stays black at `20m`. Document the remaining blocker as VirtualBox graphics instability and stop Phase 3C here.
