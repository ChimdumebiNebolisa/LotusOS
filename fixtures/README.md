# Fixtures

Small workspaces used by the testing guide and manual verification.

| Fixture | Purpose | Expected behavior |
|---|---|---|
| `demo-workspace/` | Valid workspace, one long-running process | `lotus add --trust` → start → STARTING → HEALTHY → stop → OFF |
| `invalid-manifests/cyclic-deps/` | Cyclic `depends_on` graph | `lotus add` fails: "cyclic process dependency detected" |
| `invalid-manifests/unsupported-version/` | `version = 99` | `lotus add` fails: unsupported manifest version |

The invalid fixtures are intentionally not registered by automated tests; they
let failure paths be demonstrated without hand-constructing files.
