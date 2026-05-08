# Decision 0002: Desktop Environment

## Decision

Use KDE Plasma for the first LotusOS MVP.

## Rationale

- KDE Plasma gives LotusOS a polished desktop without building a desktop environment from scratch.
- It has strong support for theming, SDDM integration, launchers, and desktop defaults.
- It better fits the first LotusOS brand target than a minimal desktop.

## Consequences

- The first ISO will be heavier than an XFCE image.
- Performance should be checked in QEMU before expanding the package set.
- Branding work should target KDE, SDDM, GRUB, and Plymouth first.

