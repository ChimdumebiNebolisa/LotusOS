# Migration: from the OS-image product to the workspace runtime

## What changed

LotusOS began as a Debian-based live/installable ISO (KDE Plasma, Calamares,
VirtualBox verification). That direction was retired in August 2026. LotusOS is
now a **local-first developer workspace runtime**: a `lotus.toml` manifest, a
process supervisor with health/port/git/checkpoint semantics, a `lotus` CLI,
and the Lotus Shell desktop app — all local, no OS image involved.

The name keeps "OS" because LotusOS operates the *context* of development work
the way an operating system schedules work — not because it ships an operating
system.

## Removed from active paths

- `os/live-build/**` — Debian live-build configuration
- Calamares installer integration and branding
- GRUB/Plymouth/SDDM/KDE packaging hooks and XDG autostart machinery
- QEMU/VirtualBox boot-verification scripts
- ISO/VDI artifacts and VM screenshots

Historical records (decision logs, per-phase VM verifications, release notes)
are preserved under [`archive/os-image-era/`](archive/os-image-era/README.md).
They describe a product that no longer exists in this repository; their
commands do not apply.

## Carried forward

- The Tauri + React + TypeScript toolchain and visual language of Lotus Shell
- The identity assets (`docs/assets/branding/`)
- The documentation discipline: claims are labeled scaffolded / built /
  booted / verified, and nothing is claimed as working without recorded
  verification

## If you used the old preview

Nothing to migrate: the old product never left disposable-VM previews. Install
the new toolchain (`cargo build`) and register any project folder containing a
`lotus.toml` as described in the README.
