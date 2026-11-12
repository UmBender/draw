---
id: ADR-T154-1
title: Every DRAW_ variable the binary reads becomes a flag
status: accepted
kind: decision
date: 2026-11-12
task: T154
builds_on: [ADR-T152-1]
amends: []
supersedes: [ADR-T152-1]
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T154-1 Every DRAW_ variable the binary reads becomes a flag

## Context

[[ADR-T152-1 Keep DRAW_TABLET as a debugging variable]] kept six `DRAW_*`
environment variables (`DRAW_LOG`, `DRAW_MSAA`, `DRAW_FRAME_TIMES`,
`DRAW_BACKEND`, `DRAW_SKIP_PREFLIGHT`, `DRAW_TABLET`) so that the command
line stayed at `--help`, `--version` and `--recover`. The owner prefers
specific commands to a polluted environment: a variable exported once
stays in force for every later run, a wrong value is not refused, and the
binary also reads `DRAW_SNAPSHOT` and three `DRAW_E2E_*` variables for the
end-to-end tests.

## Decision

- Each variable the binary reads becomes a flag with a value check; an
  unknown flag or a bad value is refused with exit code 2 and a message.
  The variables are **removed**, not kept as a fallback: one way to
  configure, and nothing is read from the environment.

  | Variable | Flag |
  |----------|------|
  | `DRAW_LOG` | `--log <off\|error\|warn\|info\|debug\|trace>` |
  | `DRAW_MSAA` | `--msaa <off\|2\|4\|8>` |
  | `DRAW_FRAME_TIMES` | `--frame-times` |
  | `DRAW_BACKEND` | `--backend <wayland\|x11>` |
  | `DRAW_SKIP_PREFLIGHT` | `--skip-preflight` |
  | `DRAW_TABLET` | `--tablet <auto\|on\|off>` |
  | `DRAW_SNAPSHOT` | `--snapshot <file>` (hidden) |
  | `DRAW_E2E_DUMP`, `DRAW_E2E_IDLE`, `DRAW_E2E_CLIPBOARD` | `--e2e-dump`, `--e2e-idle`, `--e2e-clipboard` (hidden) |

- The flags are read once at start, before the preflight and the window, and
  passed down as a typed `Options` value; no module reads the environment or
  the arguments after that. The flags that name an enum are parsed into the
  enum.
- The flags for the end-to-end tests and `--snapshot` are not in `--help` and
  not in the README's user section; `docs/features/End-to-end tests` lists
  them.
- The variables read by the test code only (`DRAW_UPDATE_SNAPSHOTS`,
  `DRAW_E2E_RUNTIME`, `DRAW_E2E_BACKEND`) stay: they configure a test run,
  not the program, and the binary never reads them.
- The README's Debugging section becomes a flags section and the guard test
  of T152 checks that no `DRAW_` name appears in `--help` or in the user
  documentation.

## Alternatives considered

- **Flags with the variables as a fallback** — two ways to set the same thing,
  and the variables keep the pollution problem.
- **Keep the variables** ([[ADR-T152-1 Keep DRAW_TABLET as a debugging variable]]) —
  a wrong value is not refused and a stale export changes later runs.

## Consequences

`DRAW_FRAME_TIMES=1 cargo run --release` becomes
`cargo run --release -- --frame-times`. Scripts, the CI workflow, the
packages and `scripts/e2e.sh` must pass the flags. Old notes and ADRs that
name the variables stay as history.

## Rollback plan

`git revert` the T154 behaviour commit and mark this ADR reverted by a new
one.
