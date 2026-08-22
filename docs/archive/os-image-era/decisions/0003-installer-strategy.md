# Decision 0003: Installer Strategy

## Decision

Use Calamares as the intended graphical installer, but do not claim LotusOS is installable until Calamares is wired and tested in a VM.

## Rationale

- Calamares is a practical installer choice for Linux distributions.
- Installer integration can become risky if attempted before the live ISO boots reliably.
- The first milestone is a bootable live ISO with LotusOS branding and Lotus Shell preinstalled or stubbed.

## Consequences

- Phase 2 may ship as live-only.
- Phase 4 owns Calamares configuration, branding, install testing, and documentation.
- README and docs must distinguish live ISO support from install support.

