# Decision 0001: Base OS

## Decision

Use Debian 13 `trixie` as the first LotusOS base and build the live ISO with Debian `live-build`.

## Rationale

- The repository had no existing Ubuntu-specific context.
- Debian stable is a conservative base for a reproducible OS image.
- `live-build` is the standard Debian tool for creating live images.
- The MVP should focus on a bootable OS before custom package infrastructure or deeper integrations.

## Consequences

- Build work should happen on Linux or WSL2, not directly in PowerShell.
- Package selection should prefer Debian repositories.
- Non-Debian package sources must be documented before use.

