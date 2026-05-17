# Lotus Shell

Lotus Shell is the planned custom app/interface layer inside LotusOS.

It is not the operating system. It should become the calm front door for AI-native student/developer workflow.

## MVP Screens

- Home.
- Projects.
- Notes.
- Files.
- AI Hub placeholder.
- Settings placeholder.

## Current Status

This directory now contains the minimal Lotus Shell Phase 3 scaffold:

- Tauri.
- React.
- TypeScript.
- Rust.
- Placeholder screens for Home, Projects, Notes, Files, AI Hub, and Settings.

The ISO build script packages the release binary into the live image at `/opt/lotus-shell/lotus-shell`. The KDE live session also includes a desktop launcher and an autostart entry so Lotus Shell opens automatically for verification.

To build Lotus Shell as part of the ISO on WSL/Linux, the host needs:

- `nodejs`
- `npm`
- `cargo`
- `rustc`
- `libgtk-3-dev`
- `libwebkit2gtk-4.1-dev`
- `librsvg2-dev`

The current Tauri stack also needs Rust Edition 2024 support, so `cargo` and `rustc` must come from a current `rustup` stable toolchain rather than an older distro package set.

## Guardrails

- Do not implement real AI integrations before the OS boots reliably.
- Do not bundle API keys.
- Do not require a cloud account for the local app shell.
