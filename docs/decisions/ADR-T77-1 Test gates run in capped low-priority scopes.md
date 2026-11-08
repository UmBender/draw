---
id: ADR-T77-1
title: Test gates run in capped low-priority scopes
status: accepted
kind: decision
date: 2026-11-08
task: T77
builds_on: []
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T77-1 Test gates run in capped low-priority scopes

## Context

`scripts/check.sh --fuzz` must leave the desktop usable on a machine with
8 threads and 7.8 GB of RAM, and must not be able to take it down
([[T77 Test sessions that leave the machine usable]]). `-j3` limits rustc
processes, not the threads inside them. Going over a memory limit has to
fail the step with a message naming it, and every step's cgroup is gone
by the time `check.sh` could look at it.

## Decision

Every `cargo` step runs as
`systemd-run --user --scope -- nice ionice cargo …`, with
`CARGO_BUILD_JOBS` and `RUST_TEST_THREADS` exported and these properties
on the scope: `MemoryMax`, `MemorySwapMax=0`, `CPUQuota`, `CPUWeight=1`
and `IOWeight=1`. The values are named at the top of the script.

**Both weights and `nice`/`ionice`.** `CPUQuota` turns the cpu controller
on along the unit's path, so the gates and the browser compete as sibling
cgroups, and `nice` only orders processes inside one cgroup. The weights
order the cgroups; `nice` and `ionice` order the step's own processes and
are all that is left where there is no scope.

**The default `OOMPolicy` reports the failure.** Each scope has a named
unit. When the kernel kills a process in it, systemd stops the whole
scope and keeps it as failed with `Result=oom-kill`. After a failed step
`check.sh` reads that with `systemctl --user show -P Result`, prints the
step and the limit, and clears the unit with `reset-failed`. It held in
25 of 25 trials, including a parent that exits as soon as its child dies.

**Without a usable `systemd-run --user`** (CI runners), a probe scope
fails once, `check.sh` prints that the memory cap is not enforced, and
the steps keep the thread limits and `nice`/`ionice`.

## Alternatives considered

- **A `--service` unit** — drops the environment (`PROPTEST_CASES`,
  `RUSTDOCFLAGS`) unless each variable is passed with `-E`.
- **`OOMPolicy=continue` and a shell inside the scope reading
  `memory.events`** — works, but adds a wrapper process, and leaves the
  rest of a dying step running.
- **`nice` and `ionice` alone** — do not rank the gates against the
  browser once the cpu controller is on.
- **`ulimit -v`** — limits address space per process, not the step, and
  breaks programs that map much more than they use.

## Consequences

- A step that outgrows 1 GB fails instead of slowing the desktop; the
  value at the top of `check.sh` is the one thing to change.
- Failed scopes do not pile up, because `check.sh` clears them.
- The probe only proves that the scope starts. A systemd that accepts
  `MemoryMax` without a delegated memory controller would not enforce it.

## Rollback plan

Remove the `gate` wrapper from `scripts/check.sh` and its tests from
`tests/ci.rs` in a `revert(T77)` commit, under a new ADR with
`kind: rollback`.
