# Phase 5A Lotus Shell Home

## Scope

Phase 5A is limited to Lotus Shell home-surface polish.

- Give `Home` a dedicated first-run dashboard layout.
- Keep Projects, Notes, Files, AI Hub, and Settings as placeholder-only surfaces.
- Keep all navigation inside Lotus Shell.
- Do not add backend storage, AI API calls, auth, cloud sync, installer changes, or ISO build changes.

## Verification

Verified locally from `shell/lotus-shell/`.

- `npm ci`
- `npm run build`
- `npm run dev`

Manual checks completed in the local Vite runtime at `http://127.0.0.1:1420/`.

- Home loads by default.
- Home heading is `A calm first place to start.`
- Hero actions are `Projects`, `Notes`, and `Files`.
- Destination cards are `Projects`, `Notes`, `Files`, `AI Hub`, and `Settings`.
- Placeholder chips are present on all destination cards.
- Navigation between Home and the placeholder sections works.
- No external links, API calls, auth, cloud sync, or data persistence were added.

## Remaining Work

Deferred beyond Phase 5A.

- Real project, notes, files, AI Hub, or settings functionality.
- Any backend, persistence, sync, or AI integration work.
- Broader Phase 5 feature expansion outside the Lotus Shell home surface.
