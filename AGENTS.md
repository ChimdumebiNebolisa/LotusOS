# AGENTS.md

## Working Agreements

- Think before coding.
- State assumptions explicitly.
- If something is unclear, say what is unclear instead of guessing.
- If multiple valid interpretations exist, present them instead of choosing silently.
- Prefer the simplest approach that fully solves the task.
- Push back on unnecessary complexity.

## LotusOS Product Guardrails

- LotusOS is a local-first developer workspace runtime: it defines, starts,
  supervises, diagnoses, stops, and restores a project's operating context as
  one workspace declared in a versioned `lotus.toml`.
- Lotus Shell is the desktop app (Tauri) frontend; the `lotus` CLI is the other
  frontend. Both must share the single core engine in `crates/lotus-core` —
  never duplicate engine behavior in either frontend.
- Do not reintroduce the retired Debian/ISO/KDE/Calamares/VM product. That era
  is archived under `docs/archive/os-image-era/` and its paths were removed.
- Do not write a custom kernel, package manager, or init system; do not build
  a desktop environment.
- The product must keep working with no AI, no cloud services, no accounts,
  and no external SaaS APIs.
- Trust is explicit: never execute commands from an untrusted or changed
  manifest; never weaken the trust gate for convenience.
- Diagnostics never expose secret values (env var names only).
- Never kill processes to free ports, and never kill by PID without verifying
  the platform identity token.

## Implementation Rules

- Write the minimum code necessary.
- Do not add features beyond what was asked.
- Do not add abstractions, configurability, or flexibility unless requested.
- Do not refactor unrelated code.
- Do not clean up adjacent code, comments, or formatting unless the task requires it.
- Match the existing style and local conventions.
- Remove only the unused imports, variables, or functions that your own changes made obsolete.
- If unrelated dead code or design issues appear, mention them instead of changing them.
- Keep all OS-specific behavior inside `lotus-core/src/platform/`; domain logic
  must not branch on the operating system.

## Execution Rules

- Turn the request into a concrete goal before coding.
- For non-trivial tasks, write a brief plan with a verification step for each major step.
- Prefer verifiable progress over broad rewrites.
- Keep build/test commands reproducible (`cargo test`, `scripts/smoke-e2e.ps1`).
- Keep generated artifacts out of source control (`target/`, Tauri `gen/`,
  logs, ledgers).

## Verification Rules

- Never claim success without verification.
- Use the narrowest reasonable check.
- Distinguish scaffolded, built, booted, and installed states where relevant.
- Windows is the verified platform; label anything else designed/pending per
  `docs/platform-support.md`.
- If something could not be verified, say that explicitly.

## Communication Rules

- Separate facts, assumptions, and interpretation.
- Surface tradeoffs early.
- Report what changed, how it was verified, and what remains uncertain.
