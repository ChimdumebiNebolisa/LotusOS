# Phase 5F: Installed System Shell Verification

Status: manually verified after an automated disposable-install attempt was blocked by a live-session lock.

## Purpose

Install the rebuilt ISO to a disposable VDI and confirm the current Lotus Shell surfaces appear after installed boot and login.

## VM Settings

- Date: `2026-05-20`
- VM name: `LotusOS-Phase5F-Installed-Test`
- Backend: `VirtualBox GUI`
- Graphics controller: `VMSVGA`
- 3D acceleration: `on`
- RAM: `4096 MB`
- CPUs: `2`
- VRAM: `128 MB`
- ISO path: `C:\Users\Chimdumebi\LotusOS\artifacts\lotusos-amd64.iso`
- Disposable VDI path: `C:\Users\Chimdumebi\VirtualBox VMs\LotusOS-Phase5F-Installed-Test\LotusOS-Phase5F-Installed-Test.vdi`
- Disposable VDI size: `25600 MBytes` capacity (`25 GiB`), dynamic VDI

## Safety Confirmation

Verified before any install step continued:

- Calamares showed exactly one target disk: `VBOX HARDDISK - 25.00 GiB (/dev/sda)`.
- No host disk, physical disk, shared-folder-backed disk, or ambiguous extra target was shown.
- The run remained confined to the disposable VM and disposable VDI created for this attempt.

## Automated Attempt Result

Live-session setup before the install attempt:

- The VM booted the rebuilt ISO and eventually recovered to KDE and Lotus Shell on the same delayed VirtualBox GUI path seen in Phase 5E.
- KDE `Welcome Center` had to be dismissed before the installer launch.
- The `Install LotusOS` launcher was found again from the KDE launcher search and Calamares opened successfully.

Installer progress reached:

- Welcome page opened.
- Location and keyboard defaults were traversed.
- The disk-selection page was reached and the disposable `/dev/sda` target was visually confirmed.

Blocker encountered:

- While navigating the partition step in the delayed live session, the desktop locked before the `Erase disk` choice and summary page could be completed cleanly.
- After the lock screen appeared, the session did not recover back to the Calamares flow in a way that allowed the disposable install to be finished reproducibly from this environment.
- Because the install did not start, the VDI remained essentially empty and no installed-system boot verification could be claimed from the automated run.

Automated-run classification:

- Automated run: `blocked during Calamares navigation because live session locked`

## Manual Follow-Up Verification

The user manually completed the missing Phase 5F verification afterward against the rebuilt ISO at `C:\Users\Chimdumebi\LotusOS\artifacts\lotusos-amd64.iso`.

Manual verification facts provided by the user:

- Live ISO boots.
- KDE/live session reaches desktop.
- Lotus Shell autostarts in the live session.
- Updated Lotus Shell Home dashboard appears.
- Settings read-only system overview appears.
- Local launcher and resource surfaces are visible and bounded.
- Calamares opens.
- Disposable VDI install flow works.
- Installed system boots from VDI.
- SDDM greeter appears.
- Login reaches Plasma desktop.
- Lotus Shell autostarts in the installed system.
- Updated shell surfaces appear in the installed system.

Manual follow-up classification:

- Manual follow-up verification: `completed by user`
- Install completed: `yes, manually verified`
- Installed system booted: `yes, manually verified`
- SDDM greeter reached: `yes`
- Plasma desktop reached: `yes`
- Lotus Shell autostarted in installed system: `yes`
- Updated Home dashboard appeared in installed system: `yes`
- Settings/system overview appeared in installed system: `yes`
- Classification: `manually verified`

## Caveats

- The blocker was in the disposable VM verification lane, not in repo code edits made during this task.
- The same VirtualBox live-session instability seen in Phase 5E affected the automated install attempt.
- No install was allowed to proceed without a visually confirmed disposable target, so the stop preserved the disk-target safety rule.
- The successful installed-system verification was manual follow-up evidence from the user, not a full end-to-end reproduction by Codex automation.

## Evidence Files

- `artifacts/vm-verification/phase5f-launcher-search4.png`
- `artifacts/vm-verification/phase5f-installer-open3.png`
- `artifacts/vm-verification/phase5f-space-next.png`
- `artifacts/vm-verification/phase5f-disk-select-after-cancel-no.png`
- `artifacts/vm-verification/phase5f-shifttab4.png`

Additional troubleshooting artifacts from the blocked run:

- `artifacts/vm-verification/phase5f-live-preinstall-10m.png`
- `artifacts/vm-verification/phase5f-live-shell-after-close.png`
- `artifacts/vm-verification/phase5f-installer-ready2.png`
- `artifacts/vm-verification/phase5f-focus-map1.png`
- `artifacts/vm-verification/phase5f-after-keyboard-next.png`
- `artifacts/vm-verification/phase5f-unlock-test.png`
- `artifacts/vm-verification/phase5f-unlock-enter-only.png`
- `artifacts/vm-verification/phase5f-unlock-progress.png`

## Result

Phase 5F is `manually verified`.

What is proven:

- The disposable install VM and disposable 25 GiB VDI were created correctly.
- Calamares launches from the rebuilt ISO.
- The installer showed only the disposable `VBOX HARDDISK - 25.00 GiB (/dev/sda)` target.
- The user manually verified that the disposable install completes on the rebuilt ISO.
- The user manually verified that the installed VDI boots, reaches SDDM and Plasma, and autostarts Lotus Shell with the updated shell surfaces visible.

What is not fully reproduced by Codex automation:

- The complete successful installed-system flow was not reproduced end to end by Codex automation because the live session locked during Calamares navigation.
