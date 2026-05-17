# Phase 3: Lotus Shell Packaging Verification

Status: completed for packaging and boot smoke test; not completed for visible desktop launch verification.

## Milestone

The smallest real Phase 3 milestone is:

- build Lotus Shell from tracked source,
- package the Linux release binary into the live ISO,
- ship desktop launcher and autostart entries in the live image, and
- confirm the rebuilt ISO still boots past the repaired BIOS GRUB path in VirtualBox.

## What Was Verified

- `shell/lotus-shell/` now contains a real Tauri + React + TypeScript scaffold instead of a placeholder-only directory.
- `os/scripts/build-iso.sh` detects Lotus Shell source, builds it from a temporary copy, and stages `/opt/lotus-shell/lotus-shell` into `config/includes.chroot/` before `lb build`.
- The final ISO was rebuilt at `artifacts/lotusos-amd64.iso`.
- The final ISO `boot/grub/grub.cfg` was extracted and confirmed to use:
  - `set root=cd0`
  - `search --no-floppy --set=root --file /live/vmlinuz-6.12.86+deb13-amd64`
  - explicit `($root)/live/...` kernel and initrd paths
- `linux.mod` exists in the final ISO at `/boot/grub/linux.mod`.
- VirtualBox BIOS boot still reaches the live boot path after the Phase 3 changes.

## What Was Not Verified

- A logged-in Plasma desktop with Lotus Shell visibly running was not proven by the VirtualBox screenshots.
- Two VirtualBox graphics-controller runs were captured:
  - `VMSVGA`: boot banner, graphical background, then unstable `vmwgfx`/black-screen states.
  - `VBoxSVGA`: boot banner and Debian 13 graphical background, but no visible Lotus Shell window by `5m`, then a text boot screen again by `10m`.

## Interpretation

The committable Phase 3 milestone is the packaging milestone, not a completed desktop-launch milestone.

The repo now produces a bootable ISO that includes a built Lotus Shell binary and the intended launcher/autostart files. The remaining blocker is live-session startup/display verification in VirtualBox: the evidence does not yet prove that the session reaches a stable logged-in desktop with Lotus Shell visible.
