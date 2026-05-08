# Build Notes

## Host Requirements

Use a Linux host or WSL2 Ubuntu. The scripts are Bash scripts and are not intended to run directly in PowerShell.

Install build tools:

```bash
sudo apt update
sudo apt install live-build qemu-system-x86 xorriso isolinux syslinux-common squashfs-tools
```

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

