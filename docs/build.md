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

Current limitation:

- This scaffold has not yet been verified to produce a successful ISO.

## QEMU Test

After an ISO exists:

```bash
bash os/scripts/test-qemu.sh artifacts/lotusos-amd64.iso
```

Expected result after Phase 2 is complete:

- QEMU starts.
- The ISO boots to a live KDE Plasma desktop.
- LotusOS branding appears where configured.

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
