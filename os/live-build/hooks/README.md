# live-build Hooks

This directory is reserved for LotusOS live-build hooks.

Planned hooks:

- Install GRUB and Plymouth branding assets.
- Install SDDM and KDE defaults.
- Install or register Lotus Shell once the Tauri app exists.

Do not add host-destructive commands here. Hooks run inside the build process and must be scoped to the image filesystem.

