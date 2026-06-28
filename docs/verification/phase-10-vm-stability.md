# Phase 10: VM Stability Verification

Status: preparation only. No Phase 10 VM result is recorded yet.

## Purpose

Phase 10 is for reducing uncertainty around LotusOS VM stability before expanding OS behavior or making stronger install claims.

This phase should focus on repeatable evidence for the supported VirtualBox lane, especially the graphics/session instability seen during earlier live and installer verification. Do not treat Phase 10 as a broad feature phase.

## Baseline Repo State

- Branch: `phase-10-vm-stability`
- Latest commit: `TBD after Phase 10 preparation commit`
- Starting commit before Phase 10 preparation: `39fa5c4` (`Add LotusOS icon and wordmark logo assets.`)
- Latest tag: `v0.1.0-preview`
- Product state: Debian 13 `trixie` live/installable ISO customization with KDE Plasma, Calamares, and packaged Lotus Shell

## Supported VirtualBox Test Lane

- OS type: Debian 64-bit
- RAM: `4096 MB`
- CPUs: `2`
- VRAM: `128 MB`
- Graphics controller: `VMSVGA`
- 3D acceleration: `on`

## Test Checklist

- [ ] ISO rebuild completed or existing ISO freshness justified
- [ ] Live boot reaches GRUB and advances into `LotusOS Live`
- [ ] Live boot reaches a usable KDE Plasma session
- [ ] SDDM/session behavior observed where applicable
- [ ] Lotus Shell autostarts in the live session
- [ ] Calamares launches from the live session
- [ ] Calamares install target is confirmed as a disposable VDI before install
- [ ] Install completes to disposable VDI
- [ ] Post-install boot succeeds after ISO detach
- [ ] Post-install login reaches Plasma
- [ ] Lotus Shell launches after installed login

## Evidence Checklist

- [ ] Screenshots captured at meaningful boot/install milestones
- [ ] VM settings recorded
- [ ] ISO path and checksum recorded
- [ ] Calamares install target confirmation captured
- [ ] Result classified as `pass`, `partial`, or `blocker`
- [ ] Remaining uncertainty recorded without overclaiming

## Known Blockers

- Transient black screen during VirtualBox live-session recovery
- `vmwgfx` or graphics console instability
- Live-session lock during Calamares navigation
- Phase 5F installed-shell verification was manually verified but not automation-reproduced
- Hardware installation remains unverified

## Result

- Date: `2026-06-27` America/Chicago
- Tester: `ChimdumebiNebolisa`
- ISO path: `artifacts/lotusos-amd64.iso`
- ISO checksum: `SHA256 499fe4671d79a99dd7838ac1d9e5fb87eb2355eb158b2e215f92e73fde0b8719`
- VM name:
- VM settings:
- Disposable VDI path:
- Live boot result:
- Lotus Shell live-session result:
- Calamares launch result:
- Install target confirmation:
- Install result:
- Post-install boot result:
- Post-install Lotus Shell result:
- Classification:
- Evidence files: `artifacts/verification/iso-contents-20260628T004851Z.txt`
- Remaining uncertainty:
