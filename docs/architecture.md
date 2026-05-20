# LotusOS Architecture

## Base System

LotusOS starts as a Debian 13 `trixie` live ISO built with `live-build`.

The first target architecture is `amd64`. The package manager is `apt`. The init system remains `systemd`.

## Desktop Layer

The first desktop target is KDE Plasma.

KDE Plasma is chosen for the preview because it provides a polished desktop, SDDM integration, theming hooks, and enough customization surface for LotusOS branding without building a new desktop environment.

## Verified Preview Shape

The current preview milestone is a Debian-based live/installable OS image that boots into KDE Plasma, autostarts Lotus Shell in the live session, and installs through Calamares onto a disposable VirtualBox VDI. Phase 5E live-session verification is complete, and Phase 5F installed-system verification is recorded as manual follow-up verification after an automated attempt was blocked by a live-session lock.

## OS Build Track

The OS build track lives under `os/`:

- `os/live-build/`: Debian `live-build` config, auto wrappers, and hooks
- `os/branding/`: source locations for GRUB, Plymouth, wallpaper, icons, SDDM, and desktop theme assets
- `os/packages/`: package list drafts and tool selection notes
- `os/scripts/`: safe entrypoints for build, QEMU test, ISO verification, and cleanup

Generated ISO, VDI, screenshot, and VM-log artifacts go under `artifacts/` and are not treated as source.

## Lotus Shell Track

Lotus Shell lives under `shell/lotus-shell/`.

The implementation track is Tauri + React + TypeScript + Rust. Its current surfaces are:

- Home with read-only session context
- Projects, Notes, and Files with bounded local resource cards
- AI Hub placeholder
- Settings with read-only local system overview

Local launcher and resource surfaces are allowlisted and local-only. Real AI integration is intentionally deferred. The AI Hub does not ship with bundled API keys or cloud-only assumptions.

The live image packages the Lotus Shell Linux binary into `/opt/lotus-shell/lotus-shell`, installs a desktop launcher, and uses an XDG autostart wrapper so the KDE session can either launch Lotus Shell or record a visible startup failure.

## Installer Track

Calamares is the graphical installer.

The live image wires Calamares through Debian's `calamares-settings-debian` package with minimal LotusOS overrides for branding, launcher text, and install confirmation behavior.

Phase 4 install verification is recorded in `docs/verification/phase-4-installer-integration.md`. Phase 5E live verification is complete on the rebuilt ISO, and Phase 5F installed verification is complete as manual user follow-up after the blocked automation attempt.

## Verification Architecture

- Build verification: `docs/verification/phase-5d-iso-rebuild.md`
- Live-session verification: `docs/verification/phase-5e-live-shell-verification.md`
- Installed-system verification: `docs/verification/phase-5f-installed-shell-verification.md`

This repo intentionally distinguishes:

- packaged into the ISO
- verified in the live session
- verified in the installed system
- manually verified versus fully reproduced by automation

## Security And Privacy Guardrails

- Local-first by default
- No bundled AI credentials
- No hidden telemetry
- No custom privileged background services before a clear need exists
- Use Debian package sources by default
- Document any non-Debian repository before adding it to the image
- Do not claim hardware installation support or production readiness without separate verification
