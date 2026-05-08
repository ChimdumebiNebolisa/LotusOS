# AGENTS.md

## Working Agreements

- Think before coding.
- State assumptions explicitly.
- If something is unclear, say what is unclear instead of guessing.
- If multiple valid interpretations exist, present them instead of choosing silently.
- Prefer the simplest approach that fully solves the task.
- Push back on unnecessary complexity.

## LotusOS Product Guardrails

- LotusOS is a full bootable Linux-based operating system image.
- Lotus Shell is the custom app/interface layer inside LotusOS.
- Do not treat Lotus Shell as the whole OS.
- Do not write a custom kernel.
- Do not create a custom package manager.
- Do not replace systemd.
- Do not build a desktop environment from scratch.
- Do not claim ISO, installer, branding, or AI functionality works until verified.

## Implementation Rules

- Write the minimum code necessary.
- Do not add features beyond what was asked.
- Do not add abstractions, configurability, or flexibility unless requested.
- Do not refactor unrelated code.
- Do not clean up adjacent code, comments, or formatting unless the task requires it.
- Match the existing style and local conventions.
- Remove only the unused imports, variables, or functions that your own changes made obsolete.
- If unrelated dead code or design issues appear, mention them instead of changing them.

## Execution Rules

- Turn the request into a concrete goal before coding.
- For non-trivial tasks, write a brief plan with a verification step for each major step.
- Prefer verifiable progress over broad rewrites.
- Keep all OS build commands reproducible.
- Keep generated artifacts out of source control unless intentionally documented.

## Verification Rules

- Never claim success without verification.
- Use the narrowest reasonable check.
- For OS work, distinguish scaffolded, built, booted, and installed states.
- If something could not be verified, say that explicitly.

## Communication Rules

- Separate facts, assumptions, and interpretation.
- Surface tradeoffs early.
- Report what changed, how it was verified, and what remains uncertain.

