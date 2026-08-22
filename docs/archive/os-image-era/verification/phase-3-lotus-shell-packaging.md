# Phase 3B: Lotus Shell Visible Launch Verification

Status: full success, with a VirtualBox graphics caveat.

## Classification

- Phase 3 result: `Full success`
- Basis: the live OS reached a visible KDE desktop state and Lotus Shell is visibly running in VirtualBox.
- Caveat: the visible-launch proof required the controlled `VMSVGA` plus 3D acceleration `on` retry; the default `VMSVGA` plus 3D acceleration `off` path still reproduces the known `vmwgfx` instability.

## Files Changed

- `os/live-build/config/includes.chroot/etc/xdg/autostart/lotus-shell.desktop`
- `os/live-build/config/includes.chroot/usr/local/bin/lotus-shell-launch`
- `docs/verification/phase-3-lotus-shell-packaging.md`

## ISO

- Path: `artifacts/lotusos-amd64.iso`
- Size: `2041642984` bytes
- Build log: `artifacts/vm-verification/phase3b-build-iso.log`

## Live Rootfs Verification

Verified from `filesystem.squashfs`, not from the ISO root:

- `/opt/lotus-shell/lotus-shell`: present, executable (`-rwxr-xr-x`)
- `/usr/share/applications/lotus-shell.desktop`: present
- `/etc/xdg/autostart/lotus-shell.desktop`: present
- `/usr/local/bin/lotus-shell-launch`: present

Desktop entry targets:

- Launcher entry: `Exec=/opt/lotus-shell/lotus-shell`
- Autostart entry: `Exec=/usr/local/bin/lotus-shell-launch`

## Autostart Logging Change

- Added `/usr/local/bin/lotus-shell-launch` to log:
  - timestamp
  - `id`
  - working directory
  - full environment
  - key display/session variables
  - `/opt/lotus-shell` listing
  - `ldd /opt/lotus-shell/lotus-shell` when available
- The autostart desktop entry now launches the wrapper instead of the binary directly.

## VirtualBox Evidence

Primary run:

- Config: `VMSVGA`, 3D acceleration `off`
- Evidence:
  - `artifacts/vm-verification/stage3b-20s.png`
  - `artifacts/vm-verification/stage3b-90s.png`
  - `artifacts/vm-verification/stage3b-5m.png`
  - `artifacts/vm-verification/stage3b-10m.png`
  - `artifacts/vm-verification/stage3b-showvminfo-before.txt`
  - `artifacts/vm-verification/stage3b-showvminfo-after.txt`
- Result:
  - boots past GRUB: yes
  - live boot starts: yes
  - graphical/wallpaper state reached: yes
  - stable desktop with Lotus Shell visible: no
  - notes: `5m` and `10m` reproduce `vmwgfx` unsupported-hypervisor errors

Controlled retry:

- Config: `VMSVGA`, 3D acceleration `on`
- Evidence:
  - `artifacts/vm-verification/stage3b-vmsvga-3d-20s.png`
  - `artifacts/vm-verification/stage3b-vmsvga-3d-90s.png`
  - `artifacts/vm-verification/stage3b-vmsvga-3d-5m.png`
  - `artifacts/vm-verification/stage3b-vmsvga-3d-10m.png`
  - `artifacts/vm-verification/stage3b-vmsvga-3d-showvminfo-before.txt`
  - `artifacts/vm-verification/stage3b-vmsvga-3d-showvminfo-after.txt`
- Result:
  - boots past GRUB: yes
  - live boot starts: yes
  - graphical state reached: yes
  - Lotus Shell visibly launched: yes
  - proof: `stage3b-vmsvga-3d-10m.png` shows a visible KDE panel and a visible `Lotus Shell` window

## Lotus Shell Launch Result

- Lotus Shell launched visibly: `yes`
- Visible evidence: `artifacts/vm-verification/stage3b-vmsvga-3d-10m.png`
- Manual launch attempt: not needed, because Lotus Shell became visible without guest interaction

## Autostart Log Result

- `/tmp/lotus-shell-autostart.log`: not collected
- Reason: no reliable guest console or terminal access was available before shutdown, and the verification goal was satisfied by visible launch evidence

## Remaining Blocker

- None for Phase 3 completion
- Caveat: the ISO is verified and Lotus Shell is visibly launching, but VirtualBox boot flow is not yet polished or reproducible across both `VMSVGA` graphics settings because `VMSVGA` with 3D acceleration `off` still falls back into the known `vmwgfx` instability path
