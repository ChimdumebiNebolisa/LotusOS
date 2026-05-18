# Phase 4: Installer Integration

Status: partial success.

## Purpose

Add a minimal, reproducible installer path to the LotusOS live ISO using Calamares without replacing Debian `live-build`, touching GRUB unnecessarily, or risking any host disk.

## Packages Added

- `calamares`
- `calamares-settings-debian`

## Config Files Added

- `os/live-build/config/includes.chroot/etc/calamares/settings.conf`
- `os/live-build/config/includes.chroot/etc/calamares/modules/welcome.conf`
- `os/live-build/config/includes.chroot/etc/calamares/branding/debian/branding.desc`
- `os/live-build/config/includes.chroot/usr/share/applications/calamares-install-debian.desktop`

The implementation reuses Debian's packaged Calamares wrapper and module set, then overrides only:

- LotusOS-visible branding strings
- launcher text
- the pre-install confirmation prompt (`prompt-install: true`)

## Launcher Path

- Live launcher path: `/usr/share/applications/calamares-install-debian.desktop`
- Live launcher command: `calamares-install-debian`

## ISO Build Result

- Result: `success`
- Build log: `artifacts/vm-verification/phase4-build-iso.log`
- ISO path: `artifacts/lotusos-amd64.iso`
- ISO size: `2465079296` bytes
- ISO timestamp: `2026-05-18 03:05:12 AM -0500`

## ISO Content Verification

Verified from the rebuilt root filesystem and build workspace:

- `calamares` binary exists
- `calamares-install-debian` wrapper exists
- `/usr/share/applications/calamares-install-debian.desktop` exists in the image
- LotusOS Calamares overrides exist under `/etc/calamares/`
- Lotus Shell and its autostart helper still exist in the image
- GRUB still auto-advances with LotusOS wording and a repaired El Torito image

Evidence:

- `artifacts/vm-verification/phase4-iso-installer-paths.txt`
- `artifacts/vm-verification/phase4-iso-grub-check.txt`

## VirtualBox Boot Verification

Primary supported lane:

- Backend: VirtualBox
- Graphics controller: `VMSVGA`
- 3D acceleration: `on`
- RAM: `4096 MB`
- CPUs: `2`
- VRAM: `128 MB`

Headless run result:

- `20s`: boot handoff visible
- `90s`: wallpaper-only graphical state
- `5m`: `vmwgfx` console error state
- `10m`: black screen

This headless result was not treated as the decisive Phase 4 live-session result because prior visible Phase 3 evidence came from the GUI frontend rather than headless.

GUI rerun result on the same VM settings and ISO:

- `20s`: `Booting 'LotusOS Live'`
- `90s`: wallpaper-only graphical state
- `5m`: KDE panel and Lotus Shell visible
- `10m`: Lotus Shell still visible

Classification from the GUI run:

- GRUB reached: yes
- GRUB auto-advanced: yes
- live boot started: yes
- KDE/live session reached: yes
- Lotus Shell still present: yes

## Installer Launch Result

- Installer packaging result: `verified`
- Launcher file result: `verified from image contents`
- Launcher visibly confirmed on-screen: `not directly proven`
- Installer opens successfully: `not directly proven`

Reason:

- In the successful GUI run, Lotus Shell autostarted and remained visible through `10m`.
- That made the live session usable enough to confirm KDE plus Lotus Shell, but the desktop area where the Calamares icon would normally appear was covered by the Lotus Shell window.
- A low-risk attempt to reveal the desktop with VirtualBox keyboard automation did not change the visible state.
- Because direct launcher visibility and direct Calamares open were not re-proven on-screen, this phase remains partial rather than full success.

## Install Test Result

- Attempted: `no`
- Reason: the installer launcher and open flow were not directly re-proven in the guest, so no disposable install disk was attached.

## Known Caveats

- The existing VirtualBox live-session caveat from Phase 3C still applies.
- The headless VirtualBox frontend fell back into a `vmwgfx` console path even with `VMSVGA` plus 3D `on`.
- The GUI frontend was materially better and stayed visibly usable through `10m`, but direct installer-launch proof is still missing.

## Safe Install Warning

- Never point Calamares at a host disk or any real user disk.
- Only test installation with a newly created disposable VM disk.
- If there is any uncertainty about which disk Calamares is targeting, stop the test.

## Evidence Files

- `artifacts/vm-verification/phase4-build-iso.log`
- `artifacts/vm-verification/phase4-iso-installer-paths.txt`
- `artifacts/vm-verification/phase4-iso-grub-check.txt`
- `artifacts/vm-verification/phase4-vmsvga-3d-showvminfo-before.txt`
- `artifacts/vm-verification/phase4-vmsvga-3d-showvminfo-after.txt`
- `artifacts/vm-verification/phase4-live-20s.png`
- `artifacts/vm-verification/phase4-live-90s.png`
- `artifacts/vm-verification/phase4-live-5m.png`
- `artifacts/vm-verification/phase4-live-10m.png`
- `artifacts/vm-verification/phase4-live-gui-20s.png`
- `artifacts/vm-verification/phase4-live-gui-90s.png`
- `artifacts/vm-verification/phase4-live-gui-5m.png`
- `artifacts/vm-verification/phase4-live-gui-10m.png`
- `artifacts/vm-verification/phase4-live-gui-post-altf4.png`
- `artifacts/vm-verification/phase4-live-gui-post-minimize-attempt1.png`
- `artifacts/vm-verification/phase4-live-gui-post-minimize-attempt2.png`

## Result

Phase 4 is a `partial success`.

What is proven:

- Calamares is integrated into the ISO with tracked LotusOS overrides.
- The ISO rebuild succeeds.
- The GUI VirtualBox path still reaches a visible KDE plus Lotus Shell state through `10m`.

What is not yet proven:

- direct on-screen visibility of the installer launcher
- direct successful opening of Calamares from the live session
- any safe disposable-disk install run
