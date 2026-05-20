# Build Notes

## Host Requirements

Use a Linux host or WSL2 Ubuntu. The scripts are Bash scripts and are not intended to run directly in PowerShell.

Install build tools:

```bash
sudo apt update
sudo apt install live-build qemu-system-x86 xorriso isolinux syslinux-common squashfs-tools
```

Lotus Shell packaging also requires these host packages on WSL/Linux:

```bash
sudo apt install nodejs npm cargo rustc libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev
```

Tauri's current Linux dependency stack requires Rust Edition 2024 support. On Debian 12 and older Ubuntu packages, the distro `cargo` and `rustc` may be too old. Load a current Rust stable toolchain from `rustup` before running the ISO build when Lotus Shell packaging is enabled.

## Dependency Check

From the repository root:

```bash
bash os/scripts/build-iso.sh --check
```

Expected result:

- The script confirms the repository layout
- The script reports whether `lb` is available
- No ISO build is started

## ISO Build

From the repository root:

```bash
bash os/scripts/build-iso.sh
```

WSL root build with a user `rustup` toolchain:

```bash
wsl.exe -u root -- /usr/bin/bash -c 'export CARGO_HOME=/home/<user>/.cargo RUSTUP_HOME=/home/<user>/.rustup PATH="/home/<user>/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"; cd /mnt/c/Users/Chimdumebi/LotusOS && bash os/scripts/build-iso.sh'
```

Expected result:

- A Debian `live-build` run starts under `os/live-build/`
- The generated ISO is copied to `artifacts/lotusos-amd64.iso`
- If Lotus Shell source is present, the build script compiles it from a temporary copy and stages the Linux Lotus Shell binary into the live image before `lb build`

Current verified state:

- The build path produces `artifacts/lotusos-amd64.iso`
- Lotus Shell packaging into the live image is verified from image contents
- GRUB wording and timeout polish are verified from the rebuilt ISO
- Calamares is packaged into the live image with tracked LotusOS overrides under `os/live-build/config/includes.chroot/etc/calamares/`
- Phase 5E live-session verification is complete on the rebuilt ISO
- Phase 5F installed-system verification is complete as manual follow-up verification after an automated Calamares run was blocked by a live-session lock
- VirtualBox verification is primarily supported with `VMSVGA` and 3D acceleration `on`
- `VMSVGA` with 3D acceleration `off` remains a known `vmwgfx` caveat

## QEMU Test

After an ISO exists:

```bash
bash os/scripts/test-qemu.sh artifacts/lotusos-amd64.iso
```

Expected result:

- QEMU starts
- The ISO boots to a live KDE Plasma desktop
- LotusOS branding appears where configured

## VirtualBox Verification

Primary supported VM mode:

- OS type: Debian 64-bit
- RAM: `4096 MB`
- CPUs: `2`
- VRAM: `128 MB`
- Graphics controller: `VMSVGA`
- 3D acceleration: `on`

Current limitation:

- VirtualBox graphics and session instability still appears in some timed runs, even in the supported `VMSVGA` + 3D `on` lane
- A minimal follow-up diagnostic that disabled `xset s off`, `xset -dpms`, and `xset s noblank` in the live session did not prevent the black-screen outcome
- This is a caveat, not a blocker to the current preview milestone documentation state

## Installer Launch And Safe Test

Current installer wiring:

- Live launcher path: `/usr/share/applications/calamares-install-debian.desktop`
- Launcher command: `calamares-install-debian`
- Tracked LotusOS overrides:
  - `os/live-build/config/includes.chroot/etc/calamares/settings.conf`
  - `os/live-build/config/includes.chroot/etc/calamares/modules/welcome.conf`
  - `os/live-build/config/includes.chroot/etc/calamares/branding/debian/branding.desc`

Safe install-testing rule:

- Never point the installer at a host disk or any real user disk
- Only test installation with a newly created disposable VM disk
- If launcher visibility or installer opening is not directly proven in the live session, stop before attaching any test disk

## ISO Content Check

After a rebuild:

```bash
wsl.exe -u root -- /usr/bin/bash os/scripts/verify-iso-contents.sh
```

## Verification References

- Phase 4 installer flow: `docs/verification/phase-4-installer-integration.md`
- Phase 5C Tauri packaging cleanup: `docs/verification/phase-5c-tauri-packaging.md`
- Phase 5D ISO rebuild: `docs/verification/phase-5d-iso-rebuild.md`
- Phase 5E live-shell verification: `docs/verification/phase-5e-live-shell-verification.md`
- Phase 5F installed-shell verification: `docs/verification/phase-5f-installed-shell-verification.md`

## WSL Root Build Note

When running the ISO build as WSL root, export the user `rustup` home directories and `PATH`, for example `CARGO_HOME=/home/<user>/.cargo`, `RUSTUP_HOME=/home/<user>/.rustup`, and `PATH="/home/<user>/.cargo/bin:..."`, so `cargo` and `rustc` meet the `>= 1.85.0` guard in `os/scripts/build-iso.sh`.

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
