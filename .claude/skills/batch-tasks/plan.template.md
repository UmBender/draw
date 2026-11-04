# Batch plan — <date>

Written in the plan phase with the owner; read by every unattended
session (`unattended.md`). The order and bases are in `.batch/queue`.

## Policies

- **Merge:** no — tasks stop at `status: review`; a task that depends on
  another task in this batch branches from that task's branch.
- **Push:** no.
- **Open choices** (small decisions the plan does not cover):
  `decide-and-record` — pick the recommended option and record it as an
  ADR or a `UNATTENDED CHOICE:` *Log* line — or `block`.
- **Time:** stop starting new attempts after <time or "none">
  (`BATCH_STOP_AT`).

## Prerequisites (done by the owner before `run`)

- [ ] System packages: <e.g. `sudo dnf install sway grim wtype` or none>
- [ ] Logged in to Claude Code (`claude` works), network up.
- [ ] Working tree clean on `main`.
- [ ] `batch.sh check` green.

## Order

| # | Task | Base | Depends on | Expected | Risk of blocking |
|---|------|------|------------|----------|------------------|
| 1 | T… | main | — | <size> | <low/medium/high: why> |

## T… <title>

- **Owner decisions:** <answers to the questions asked in the plan phase>
- **Recommended defaults** for choices likely to come up: <…>
- **Commands and tools** beyond the base allowlist: <command → why>
- **Block if:** <task-specific conditions, e.g. needs a visual check>
- **Manual checks left for the owner:** <…>
