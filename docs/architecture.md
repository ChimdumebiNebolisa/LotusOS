# LotusOS Architecture

## Base System

LotusOS starts as a Debian 13 `trixie` live ISO built with `live-build`.

The first target architecture is `amd64`. The package manager is `apt`. The init system remains `systemd`.

## Desktop Layer

The first desktop target is KDE Plasma.

KDE Plasma is chosen for the MVP because it provides a polished desktop, strong theming hooks, SDDM integration, and enough customization surface for LotusOS branding without building a new desktop environment.

## OS Build Track

The OS build track lives under `os/`:

- `os/live-build/`: Debian `live-build` config, auto wrappers, and hooks.
- `os/branding/`: source locations for GRUB, Plymouth, wallpaper, icons, SDDM, and desktop theme assets.
- `os/packages/`: package list drafts and tool selection notes.
- `os/scripts/`: safe entrypoints for build, QEMU test, and cleanup.

Generated ISO artifacts should go under `artifacts/` and should not be treated as source.

## Lotus Shell Track

Lotus Shell lives under `shell/lotus-shell/`.

The implementation track is Tauri + React + TypeScript + Rust. Its MVP screens are:

- Home with read-only session context.
- Projects, Notes, and Files with bounded local resource cards.
- AI Hub placeholder.
- Settings with read-only local system overview.

Local launcher and resource surfaces are allowlisted and local-only. Real AI integration is planned after boot and install reliability. The AI Hub must not ship with bundled API keys or cloud-only assumptions.

The live image currently packages the Lotus Shell Linux binary into `/opt/lotus-shell/lotus-shell`, installs a desktop launcher, and uses an XDG autostart wrapper so the KDE live session can either launch Lotus Shell or record a visible/logged startup failure.

The remaining Phase 3C blocker is not Lotus Shell packaging. The unresolved issue is live-session stability under the supported VirtualBox `VMSVGA` plus 3D `on` lane.

## Installer Track

Calamares is the intended graphical installer.

The live image now wires Calamares through Debian's `calamares-settings-debian` package with minimal LotusOS overrides for branding, launcher text, and install confirmation behavior.

Phase 4 install verification is recorded in `docs/verification/phase-4-installer-integration.md`. The remaining image-level gap is rebuilding the ISO with the current Lotus Shell and re-verifying live and installed sessions after that rebuild.

## Security And Privacy Guardrails

- Local-first by default.
- No bundled AI credentials.
- No hidden telemetry.
- No custom privileged background services before a clear need exists.
- Use Debian package sources by default.
- Document any non-Debian repository before adding it to the image.
