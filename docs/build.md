# Build Notes

## Host Requirements

Use a Linux host or WSL2 Ubuntu. The scripts are Bash scripts and are not intended to run directly in PowerShell.

Install build tools:

```bash
sudo apt update
sudo apt install live-build qemu-system-x86 xorriso isolinux syslinux-common squashfs-tools
```

Phase 3 Lotus Shell packaging also requires these host packages on WSL/Linux:

```bash
sudo apt install nodejs npm cargo rustc libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev
```

Tauri's current Linux dependency stack requires Rust Edition 2024 support. On Debian 12 / Ubuntu packages, the distro `cargo`/`rustc` may be too old. Load a current Rust stable toolchain from `rustup` before running the ISO build when Lotus Shell packaging is enabled.

## Dependency Check

From the repository root:

```bash
bash os/scripts/build-iso.sh --check
```

Expected result:

- The script confirms the repository layout.
- The script reports whether `lb` is available.
- No ISO build is started.

## ISO Build

From the repository root:

```bash
bash os/scripts/build-iso.sh
```

Expected result after Phase 2 is complete:

- A Debian live-build run starts under `os/live-build/`.
- The generated ISO is copied to `artifacts/lotusos-amd64.iso`.
- If Lotus Shell source is present, the build script compiles it from a temporary copy and stages the Linux Lotus Shell binary into the live image before `lb build`.

Current verified state:

- The build path produces a successful ISO.
- Lotus Shell packaging into the live image is verified from the build and include paths.
- GRUB wording and timeout polish are verified from the rebuilt ISO.
- VirtualBox verification is primarily supported with `VMSVGA` and 3D acceleration `on`.
- `VMSVGA` with 3D acceleration `off` remains a known `vmwgfx` caveat.
- Phase 3C implementation exists, but stable desktop acceptance is still not re-proven.

## QEMU Test

After an ISO exists:

```bash
bash os/scripts/test-qemu.sh artifacts/lotusos-amd64.iso
```

Expected result after Phase 2 is complete:

- QEMU starts.
- The ISO boots to a live KDE Plasma desktop.
- LotusOS branding appears where configured.

## VirtualBox Verification

Primary supported VM mode:

- OS type: Debian 64-bit
- RAM: `4096 MB`
- CPUs: `2`
- VRAM: `128 MB`
- Graphics controller: `VMSVGA`
- 3D acceleration: `on`

Evidence saved for the current Phase 3C pass:

- `artifacts/vm-verification/phase3c-build-iso.log`
- `artifacts/vm-verification/phase3c-vmsvga-3d-20s.png`
- `artifacts/vm-verification/phase3c-vmsvga-3d-90s.png`
- `artifacts/vm-verification/phase3c-vmsvga-3d-5m.png`
- `artifacts/vm-verification/phase3c-vmsvga-3d-10m.png`
- `artifacts/vm-verification/phase3c-vmsvga-3d-current.png`
- `artifacts/vm-verification/phase3c-vmsvga-3d-final.png`
- `artifacts/vm-verification/phase3c-vmsvga-3d-final-unlocked.png`
- `artifacts/vm-verification/phase3c-vmsvga-3d-showvminfo-before.txt`
- `artifacts/vm-verification/phase3c-vmsvga-3d-showvminfo-after.txt`
- `artifacts/vm-verification/phase3c-keepawake-build-iso.log`
- `artifacts/vm-verification/phase3c-keepawake-vmsvga-3d-5m.png`
- `artifacts/vm-verification/phase3c-keepawake-vmsvga-3d-10m.png`
- `artifacts/vm-verification/phase3c-keepawake-vmsvga-3d-15m.png`
- `artifacts/vm-verification/phase3c-keepawake-vmsvga-3d-20m.png`
- `artifacts/vm-verification/phase3c-keepawake-vmsvga-3d-showvminfo-before.txt`
- `artifacts/vm-verification/phase3c-keepawake-vmsvga-3d-showvminfo-after.txt`

Current limitation:

- Lotus Shell is visibly launchable under `VMSVGA` with 3D acceleration `on`, but the session still later degrades to a black screen.
- A minimal follow-up diagnostic that disabled `xset s off`, `xset -dpms`, and `xset s noblank` in the live session did not prevent the black-screen outcome.
- The remaining blocker is VirtualBox graphics and session stability, not Lotus Shell packaging.

## Cleanup

Preview cleanup:

```bash
bash os/scripts/clean.sh --dry-run
```

Clean generated build outputs:

```bash
bash os/scripts/clean.sh
```

The clean script is intentionally scoped to generated paths inside this repository.
