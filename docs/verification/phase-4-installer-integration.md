# Phase 4: Installer Integration

Status: full Phase 4 success; disposable install and installed desktop boot verified.

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

GUI follow-up proof on the same VM settings and existing ISO:

- `20s`: graphical wallpaper state visible
- `90s`: transient `vmwgfx` console text visible
- `5m`: KDE panel and Lotus Shell visible
- manual launch path used: KDE application launcher search for `install lotusos`

## Installer Launch Result

- Installer packaging result: `verified`
- Launcher file result: `verified from image contents`
- Launcher visibly confirmed on-screen: `verified`
- Installer opens successfully: `verified`
- Branding result: `LotusOS Installer` window title and `Welcome to the Calamares installer for LotusOS Preview`
- Install started: `no`
- Disk modified: `no`

Proof path:

- Lotus Shell autostarted and covered the desktop, so the KDE application launcher was opened from the panel in the live session.
- Searching for `install lotusos` showed the `Install LotusOS` launcher on-screen.
- Selecting that launcher opened the Calamares welcome window in the live session.
- No disposable install disk was attached, so Calamares later reported that installation could not continue and that there were no partitions to install on.
- That warning was treated as confirmation that no install target was presented, not as an installer-open failure.

## Install Test Result

- Attempted: `yes`
- Installer completion: `yes`
- Installed-system boot result: `verified`
- Greeter reached: `yes`
- Desktop reached: `yes`
- Lotus Shell autostart in installed system: `yes`
- Classification: `Full Phase 4 success`

## Phase 4C Safe Disposable-Disk Install Test

- Date/time: `2026-05-19 18:01 -05:00` to `2026-05-19 19:12 -05:00`
- VM name: `LotusOS-Phase4C-Install-Test`
- Disposable VDI path: `C:\Users\Chimdumebi\VirtualBox VMs\LotusOS-Phase4C-Install-Test\LotusOS-Phase4C-Install-Test.vdi`
- Disposable VDI size: `25600 MBytes` capacity (`25.00 GiB`), dynamic VDI, `8441 MBytes` on disk after the test
- ISO used: `artifacts/lotusos-amd64.iso`
- Live-session launcher path used: KDE application launcher search for `install lotusos`

Disk-target safety confirmation:

- Calamares showed exactly one install target: `VBOX HARDDISK - 25.00 GiB (/dev/sda)`
- No host disk, physical disk, shared-folder-backed disk, or other ambiguous disk target was shown
- The summary page matched the same disposable target and showed `Erase disk /dev/sda (VBOX HARDDISK)` with boot loader install on `/dev/sda`

Install execution result:

- Calamares install started only after the disposable-disk target was visually confirmed
- Calamares reached the finish page and reported `All done.`
- The live session blanked and locked during the long install run, but the installer remained active and later reached the finish screen

Installed-boot result after ISO detach:

- VM was powered off
- ISO was detached
- Boot order was switched to hard disk only
- VirtualBox BIOS reported `Booting from Hard Disk...`
- Installed VDI booted successfully after ISO detach
- SDDM greeter appeared for `Lotus Tester`
- Login succeeded as user `lotus`
- Plasma `(Wayland)` desktop loaded
- Lotus Shell autostarted inside the installed system

## Known Caveats

- The existing VirtualBox live-session caveat from Phase 3C still applies.
- The headless VirtualBox frontend fell back into a `vmwgfx` console path even with `VMSVGA` plus 3D `on`.
- The GUI frontend was materially better and reached a usable live session again, but timed screenshots still showed an intermittent transient `vmwgfx` console frame before the `5m` live-session state.
- Long-term hardware compatibility and broader VM/backend coverage remain future work beyond Phase 4 verification.

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
- `artifacts/vm-verification/phase4-installer-open-20s.png`
- `artifacts/vm-verification/phase4-installer-open-90s.png`
- `artifacts/vm-verification/phase4-installer-open-5m.png`
- `artifacts/vm-verification/phase4-installer-open-launcher.png`
- `artifacts/vm-verification/phase4-installer-open-search.png`
- `artifacts/vm-verification/phase4-installer-open.png`
- `artifacts/vm-verification/phase4-installer-post-close.png`
- `artifacts/vm-verification/phase4c-installer-disk-selection.png`
- `artifacts/vm-verification/phase4c-installer-summary-before-install.png`
- `artifacts/vm-verification/phase4c-install-progress.png`
- `artifacts/vm-verification/phase4c-install-complete.png`
- `artifacts/vm-verification/phase4c-install-after-unlock-53m.png`
- `artifacts/vm-verification/phase4c-installed-boot-90s.png`
- `artifacts/vm-verification/phase4c-installed-boot-5m.png`
- `artifacts/vm-verification/phase4c-installed-boot-10m-check.png`
- `artifacts/vm-verification/phase4c-installed-greeter.png`
- `artifacts/vm-verification/phase4c-installed-login-transition.png`
- `artifacts/vm-verification/phase4c-installed-desktop-lotus-shell.png`

## Result

Phase 4 is `full success`.

What is proven:

- Calamares is integrated into the ISO with tracked LotusOS overrides.
- Calamares opens from the live session.
- Calamares targets only the disposable `25.00 GiB` VDI.
- The install completes successfully.
- The installed system boots from the disposable VDI.
- SDDM greeter appears.
- Login reaches the Plasma desktop.
- Lotus Shell autostarts in the installed system.

What is not yet proven:

- No critical Phase 4 blocker remains.
- Long-term hardware compatibility remains future work.

Phase 4 can now be treated as complete for installer integration verification.
