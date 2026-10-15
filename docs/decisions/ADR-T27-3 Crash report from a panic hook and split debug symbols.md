---
id: ADR-T27-3
title: Crash report from a panic hook and split debug symbols
status: accepted
kind: decision
date: 2026-10-15
task: T27
builds_on: [ADR-0002, ADR-0014]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T27-3 Crash report from a panic hook and split debug symbols

## Context

The release profile has `panic = "abort"` and `strip = true`: small and
fast, but a crash on a user's machine leaves only "Aborted". To trace a
report back to the source we need the panic's message and location, the
exact build, and symbols that are not shipped in the ~1 MB binary.

## Decision

- **Panic hook.** `main` installs `shell::crash::install_hook` first.
  The hook writes one report — message, `file:line:column`, crate
  version, GNU build id, Unix time, and a backtrace when
  `RUST_BACKTRACE` asks for one — to stderr and appends it to
  `$XDG_STATE_HOME/draw/crash.log` (fallback `~/.local/state/draw/`).
  `panic = "abort"` stays: the hook runs before the abort. Formatting,
  the log path and reading the build id from `/proc/self/exe` (an ELF
  `PT_NOTE` walk over bytes) are pure functions.
- **Debug symbols.** A `dist` profile inherits `release` with
  `debug = true` and `strip = false`. `scripts/package.sh` runs
  `objcopy --only-keep-debug` to write `draw.debug`, strips the shipped
  binary (`--strip-all`) and adds a `.gnu_debuglink` to `draw.debug`.
  Both keep the linker's build id (`-Wl,--build-id` is passed
  explicitly). `draw.debug` goes to `dist/` beside the packages, not
  into them.
- A crash address from a core dump (`coredumpctl info`, which prints the
  build id) is resolved with
  `addr2line -e draw.debug <offset>`; a panic report names the source
  line directly.

## Alternatives considered

- **`panic = "unwind"`** — larger binary and slower code for a report
  the hook gives anyway.
- **Ship symbols in the binary** — 10× the size for every user, to
  help the few who crash.
- **A crash-reporting dependency** — the app keeps its dependencies
  ([[ADR-0014 Minimal dependencies and no unsafe]]).

## Consequences

- Every crash leaves a log a user can paste into an issue.
- The release process must keep `draw.debug` for every published build
  id; `SHA256SUMS` lists it.
- The hook must not panic itself: every step falls back silently.

## Rollback plan

Remove the `install_hook` call and the `dist` profile; packages then
build with the `release` profile again.
