# Platform Support

V1 is **Windows-first**. All platform-specific behavior is isolated in
`lotus-core/src/platform/{windows.rs,unix.rs}`; domain logic contains no OS
conditionals.

## Matrix (verified vs designed)

| Capability | Windows 10/11 | Linux | macOS |
|---|---|---|---|
| Manifest parsing/validation | verified | designed¹ | designed¹ |
| Trust store / registry | verified | designed¹ | designed¹ |
| Process spawn + supervision | verified | designed (process groups) | designed |
| Dependency ordering, restart policy | verified (tests) | designed¹ | designed¹ |
| Health: TCP/HTTP/path/command | verified (tests) | designed¹ | designed¹ |
| Graceful stop (SIGTERM) | n/a — see below | designed | designed |
| Forced tree termination | verified (taskkill /T /F) | designed (killpg) | designed |
| Port owner discovery | verified (netstat+tasklist) | via `lsof` or `ss` | via `lsof` |
| PID identity tokens (anti-PID-reuse) | verified (creation FILETIME) | `/proc/<pid>/starttime` | NOT available — cleanup skips unverifiable PIDs by design |
| Git context | verified | same code path | same code path |

¹ "designed" = implemented behind the platform adapter with cross-platform
std/libc APIs and covered by platform-independent tests where possible, but
**not executed on that OS during this build**. Per repo rules these are not
claimed as verified. CI on the target OS would upgrade them.

## Windows specifics

- Executable resolution honors PATHEXT-style extensions (.exe/.bat/.cmd/.com);
  batch files are launched through `cmd /C`.
- There is no SIGTERM for console applications. "Graceful" stop is a
  non-forced `taskkill` attempt (only effective for GUI apps), followed by the
  grace-period wait, then `taskkill /T /F`. In practice Windows stops are
  forced after grace; manifests should keep `grace_secs` modest.
- Detached supervisors are created with `DETACHED_PROCESS |
  CREATE_NEW_PROCESS_GROUP` and null stdio.
- Interactive shells that wait on inherited output handles may appear to hold
  while a detached supervisor runs; redirect output in scripts.

## Unix specifics

- Children get a new session/process group (`process_group(0)`), so tree
  signals use `killpg`; graceful = SIGTERM, forced = SIGKILL.
- Port discovery prefers `lsof -nP -iTCP -sTCP:LISTEN`, falls back to
  `ss -ltnpH` on Linux; without either, doctor reports port checks UNVERIFIED
  and pre-flight conflicts are skipped with a ledger event (startup still
  proceeds — diagnostics are advisory).
- PID identity tokens exist on Linux only; on other Unixes orphan cleanup
  refuses to kill PIDs it cannot verify (reported, never guessed).

## Adding a platform

Implement the function set in a new `platform/<os>.rs` module and re-export it
from `platform/mod.rs`. Nothing outside that module may branch on `cfg!(...)`.
