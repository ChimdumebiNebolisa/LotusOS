# LotusOS Baseline Audit (Rebuild Baseline)

- Audit date: 2026-08-21
- Auditor: ox-alpha (adversarial protocol: `ADVERSARIAL_CODEBASE_AUDIT_PROTOCOL.md`)
- Repository: `ChimdumebiNebolisa/LotusOS` (https://github.com/ChimdumebiNebolisa/LotusOS.git)
- Local path: `C:\Users\Chimdumebi\LotusOS`
- Branch at audit time: `phase-10-vm-stability`
- HEAD at audit time: `579dbf902ac2c7bdeb404f1d69e08c78c7e11c2b`
- Working tree at audit time: clean (`git status --short` empty)
- Default branch: `main` (= `39fa5c4`, 6 commits behind the audit branch)
- Submodules: none. Git LFS: not in use. Tags: `v0.1.0-preview`.
- Toolchain: Windows 11 (10.0.26200, AMD64), Node v24.14.1, npm 11.11.0, rustc/cargo 1.94.1, git 2.51.0.windows.1
- Authorization context: read-only baseline audit; a major product rebuild is explicitly authorized as a follow-on task with a locked direction (workspace runtime). This report records the state being replaced.

## AUDIT COMPLETE WITH LIMITATIONS

Reasons: no CI exists to inspect; ISO/VM verification claims could not be re-run (require Linux/WSL live-build and VirtualBox and are out of scope for a Windows-host read-only pass); the Tauri app was exercised via build/check only, not interactively launched. All other applicable domains were inspected directly.

## Executive verdict

**Do not ship** (as currently positioned). The repository presents two incompatible products: documentation claims a bootable Debian/KDE/Calamares OS image, while the only code that runs on the developer's actual platform (Windows) is a Tauri shell whose backend reports `Unknown` for nearly every field because its system snapshot logic reads Linux-only paths (`/etc/os-release`, `/proc/cmdline`). The shell contains four placeholder sections including an "AI Hub" that contradicts the local-first scope. There are zero automated tests, zero CI workflows, and the entire product surface is one 1075-line React component plus one 513-line Rust file. Overall codebase risk: **High**. Audit confidence: **High** (codebase is small enough for full inspection).

Top risks:

1. F-001 Product identity contradiction: docs promise an OS image; the runnable artifact is a desktop app with degraded data on Windows.
2. F-002 Zero tests anywhere in the tree.
3. F-003 Linux-only assumptions baked into every backend command; Windows gets silent `Unknown` values instead of honest unsupported states.
4. F-004 Placeholder sections (Projects/Notes/Files/AI Hub) shipped as UI with no backing behavior.
5. F-005 Duplicated function definition in `build-iso.sh` silently overrides the first definition.

Strongest counterargument to the verdict: as a *documentation-of-a-Linux-project* milestone the repo is internally consistent — the ISO path was genuinely verified per docs, screenshots are committed as evidence, and claims are carefully hedged ("preview", "manually verified"). That defense does not change the finding for the Windows-first rebuild decision, but it explains why the old repo is not fraudulent, merely obsolete relative to the locked new direction. What the team is most likely underestimating: how much of the existing shell (styles, command-boundary pattern, PATH-resolution helper) survives into the new architecture, versus how little of it (distro snapshot, KDE launchers, Calamares gating) does.

Immediate next action: execute the authorized rebuild (Phases B–D) using this report as the salvage baseline.

## Coverage ledger

92 tracked files total. Classification:

| Path group | Files | Depth | Status | Notes |
|---|---:|---|---|---|
| `README.md`, `LICENSE`, `.gitignore`, `.gitattributes`, `AGENTS.md` | 5 | Full | Inspected | F-006 |
| `docs/architecture.md`, `vision.md`, `roadmap.md`, `planning/lotusos-master-plan.md`, `decisions/*` | 8 | Full | Inspected | F-001 |
| `docs/verification/*`, `docs/releases/*` | 13 | Skim + spot-check | Metadata/partial | Historical evidence for old product; archived during rebuild |
| `shell/.../src/App.tsx` | 1 | Full | Inspected | F-003, F-004 |
| `shell/.../src/main.tsx`, `index.html`, `tsconfig*.json`, `vite.config.ts` | 5 | Full | Inspected | F-007 |
| `shell/.../src/styles.css` | 1 | Structural review | Inspected | Design tokens reusable |
| `shell/.../src-tauri/src/main.rs` | 1 | Full | Inspected | F-003 |
| `shell/.../src-tauri/{Cargo.toml,build.rs,tauri.conf.json,capabilities}` | 4 | Full | Inspected | Clean scaffold |
| `shell/.../package.json`, lockfile | 2 | Manifest review | Inspected | Lockfile present, reproducible install assumed (cache reused) |
| `shell/.../src-tauri/gen/schemas/*.json` | 4 | Generated | Generated | Checked into git; identical desktop/linux schemas duplicated |
| `shell/.../icons/*` | 2 | Binary | Binary | Fine |
| `os/live-build/**`, `os/scripts/*.sh`, `os/packages/*` | 25 | Targeted full reads (`build-iso.sh`, `auto/config`, package list) + structural scan | Partial | Old product machinery; F-005 |
| `os/branding/logos/*.png`, other branding READMEs | 9 | Provenance check | Partial | Wordmark/icon identity assets have reuse value |
| `artifacts/vm-verification/*.png` | 7 | Provenance check | Binary/metadata | Tracked despite `/artifacts/` ignore rule (F-008) |

Critical paths received full coverage: yes (the entire active application is 2 source files).

## Build and verification results

| Check | Command | Result | Key output | Interpretation |
|---|---|---|---|---|
| Type-check + web build | `npm run build` (tsc && vite build) | Pass | 33 modules, built in 3.49s | TS compiles clean |
| Rust check | `cargo check` (src-tauri) | Pass | Finished dev profile in 2m05s | Compiles on Windows |
| Unit tests | none exist | N/A | No test files found (`*.test.*` count = 0; no `#[cfg(test)]`) | F-002 |
| Lint | none configured | N/A | No eslint/rustfmt/clippy config | Quality gates absent |
| Runtime smoke | not run | Blocked (out of scope) | — | Launch behavior unverified this session |

## Findings summary

| ID | Severity | Priority | Confidence | Category | Title | Status |
|---|---|---|---|---|---|---|
| F-001 | High | P1 | High | Product | Repo presents itself as a Debian ISO product; runnable code is only a shell app | Verified |
| F-002 | High | P1 | High | Testing | Zero automated tests across all 92 files | Verified |
| F-003 | High | P1 | High | Correctness | Backend commands are Linux-only; on Windows they return `Unknown`/false silently | Verified |
| F-004 | Medium | P2 | High | Product | Four placeholder sections incl. AI Hub ship as interactive UI without behavior | Verified |
| F-005 | Low | P3 | High | Correctness | `ensure_grub_menu_defaults` defined twice in `build-iso.sh`; first copy is dead | Verified |
| F-006 | Low | P3 | High | Documentation | README license link is an absolute local path (`C:/Users/...`) | Verified |
| F-007 | Low | P3 | High | Security/hygiene | Vite dev/preview servers bind `0.0.0.0` (network-exposed dev server) | Verified |
| F-008 | Informational | P3 | High | Hygiene | `artifacts/vm-verification/*.png` tracked although `.gitignore` ignores `/artifacts/` | Verified |
| F-009 | Informational | P3 | High | Hygiene | Generated Tauri schemas (incl. byte-identical linux-schema duplicate) checked into git | Verified |

## Detailed findings

### F-001: Two incompatible product identities in one repository
- **Severity/Priority/Confidence:** High / P1 / High — **Status:** Verified
- Evidence: `README.md:5` ("custom Debian-based Linux live/installable ISO ... verified on disposable VirtualBox VDIs"); `docs/architecture.md:5-17`; `docs/vision.md:17-20` ("LotusOS is the full bootable OS image"); while `shell/` is the only executable code and `os/` is inert configuration/scripts requiring WSL+root+VirtualBox.
- Impact: any consumer cannot tell what the product is; setup instructions (WSL live-build) do not apply to the shell app that actually builds on Windows.
- Remediation: executed by the authorized rebuild — rewrite active docs around the workspace-runtime product, archive old material.

### F-002: Zero automated tests
- **Severity/Priority/Confidence:** High / P1 / High — **Status:** Verified
- Evidence: no `#[cfg(test)]`/`#[test]` in any `.rs` file; zero `*.test.ts(x)` files; no test script in `package.json`; README "Testing" section lists only build commands.
- Impact: every claim ("verified", "works") rests on manual VM observation recorded in prose; regressions are undetectable.
- Remediation: rebuild introduces a tested core crate; adversarial matrix from the rebuild prompt drives test list.

### F-003: Silent degradation on non-Linux platforms
- **Severity/Priority/Confidence:** High / P1 / High — **Status:** Verified
- Evidence: `main.rs:384-390` (`detect_live_session` reads `/run/live/medium`, `/proc/cmdline`), `main.rs:424-452` (`system_snapshot` reads `/etc/lotusos-release`, `/etc/os-release`, `/etc/hostname`); launcher candidates are KDE apps (`konsole`, `dolphin`, ...). On Windows every field becomes `"Unknown"` and `has_calamares_launcher=false`; UI then renders misleading "Installed system" mode text (`App.tsx:288-296` maps unknown→"Unknown session" but `session_mode` computes to `"installed"` since detection just returns false).
- Reproduction: run the packaged shell on Windows → Settings shows Base system "Unknown", Session mode "Installed system".
- Impact: the app's primary feature (truthful local context) is false on the platform where development happens.
- Remediation: new core has explicit platform adapter; unsupported capability surfaces as "unsupported", never as fabricated defaults.

### F-004: Placeholder sections presented as product surface
- **Severity/Priority/Confidence:** Medium / P2 / High — **Status:** Verified
- Evidence: `App.tsx:78-127` (sections Projects/Notes/Files/AI Hub with `statusLabel: "Placeholder"`), rendered as clickable destinations (`App.tsx:621-636`). Honestly labeled, but they are dead-end navigation occupying the primary IA.
- Impact: dilutes the real value; AI Hub contradicts "product must work without AI" scoping in the new direction.
- Remediation: removed in rebuild; navigation rebuilt around workspaces.

### F-005: Duplicate bash function definition
- **Severity/Priority/Confidence:** Low / P3 / High — **Status:** Verified
- Evidence: `os/scripts/build-iso.sh:166` and `os/scripts/build-iso.sh:245` both define `ensure_grub_menu_defaults`; bash silently uses the last definition. The first (simple timeout/default enforcement) is dead code. Moot once `os/` retires; recorded for completeness.

### F-006: Broken license link
- **Severity/Priority/Confidence:** Low / P3 / High — **Status:** Verified
- Evidence: `README.md:189`: `[LICENSE](C:/Users/Chimdumebi/LotusOS/LICENSE)` resolves only on one machine. Fix in doc rewrite.

### F-007: Dev server binds all interfaces
- **Severity/Priority/Confidence:** Low / P3 / High — **Status:** Verified
- Evidence: `vite.config.ts` sets `host: "0.0.0.0"` for dev and preview; `tauri.conf.json:7` passes `--host 0.0.0.0`. Exposes the HMR/dev server on LAN during development. Low risk (local dev tooling) but unjustified default. Fixed by binding loopback in rebuild.

### F-008/F-009: Hygiene contradictions
- **Severity/Priority/Confidence:** Informational / P3 / High — **Status:** Verified
- Evidence: `.gitignore` line 2 ignores `/artifacts/` yet 7 PNGs under `artifacts/vm-verification/` are tracked (added before ignore rule or force-added). `src-tauri/gen/schemas/linux-schema.json` is byte-identical to `desktop-schema.json` and both are generated artifacts under version control.
- Impact: minor; signals inconsistent artifact policy. Addressed by rebuild's documented artifact rules.

## Contradictions table

| ID | Source A | Source B | Contradiction | Runtime authority |
|---|---|---|---|---|
| C-1 | `docs/vision.md:18` ("full bootable OS image") | Rebuild mandate (task prompt) | Product definition changed; docs must be rewritten | Task prompt governs going forward |
| C-2 | `README.md` "Testing" section | Reality | Lists manual VM steps as testing; no automated tests exist | Code |
| C-3 | `.gitignore` `/artifacts/` | Tracked files under `artifacts/` | Ignore rule not effective for already-tracked files | Git index |
| C-4 | `AGENTS.md` guardrail "Lotus Shell is not the whole OS" | Locked new direction makes Lotus Shell/the app the whole product surface | Direction supersedes old guardrail wording; AGENTS.md updated in rebuild | Task prompt |

## Rejected hypotheses

- "The shell fabricates data everywhere": rejected as blanket claim — the fallback preview path (`isTauri()` false → static preview objects) is honestly labeled; fabrication is limited to the Linux-path-miss cases (F-003).
- "Secrets or credentials in tree": rejected — searched password/secret/token/key patterns across shell sources; none present. Package lists contain no credential material.
- "`npm ci` irreproducible": not fully verifiable this session (reused cache), but lockfile is present and consistent with manifest; residual uncertainty noted.
- "Live-build scripts dangerous if run": reviewed `build-iso.sh` — guarded build root under `/tmp/lotusos-*`, requires root explicitly, refuses non-Linux hosts. Safe by design (and being retired anyway).

## Salvage analysis for the rebuild

Salvage (verified reusable):

1. Tauri scaffold: `Cargo.toml`, `build.rs`, `tauri.conf.json` structure, capabilities pattern, icons (`icon.ico/png`).
2. Frontend toolchain: Vite + React 18 + TS config; `package-lock.json`.
3. Visual design language: `styles.css` panel/badge/fact-grid primitives fit the new workspace UI.
4. Patterns worth porting: `resolve_command_on_path` (PATH resolution → doctor), `parse_release_file`/env helpers (general shape), serde camelCase command boundary, `isTauri()` browser-preview honesty pattern.
5. Identity assets: `os/branding/logos/*` wordmark/icon (unique copies), `docs/assets/readme/lotus-shell-home.png` style references.

Retire (no salvage): `SystemSnapshot` distro logic, KDE launcher/resource tables, live-session detection, all `os/live-build` machinery, Calamares configs, GRUB/QEMU/VirtualBox verification lanes, autostart wrappers.

Archive (historical value): phase verification docs and master plan move to `docs/archive/os-image-era/` with a README stating they describe the retired Debian/ISO product.

## Residual unknowns

- Whether `npm ci` completes from a cold cache (not exercised; low risk).
- Packaged-app runtime behavior on macOS/Linux (never claimed; new platform support doc will scope V1 to Windows).
- Exact VirtualBox instability root cause (documented by prior phases; irrelevant after retirement).

— End of baseline audit. Modification authorization begins after this point.
