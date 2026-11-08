---
id: T77
title: Test sessions that leave the machine usable
status: done
wave: 33
branch: task/T77-gate-resource-limits
depends_on: []
adrs: [ADR-T77-1]
feature: ""
tutorial: ""
tags: [task]
---

# T77 Test sessions that leave the machine usable

## Goal

The owner can keep using the browser during a whole
`scripts/check.sh --fuzz`. Every step uses at most 3 threads and 1 GB of
RAM, yields to the desktop when it wants the CPU or the disk, and can no
longer take the machine down: going over a limit fails that step with a
clear message. The thirstiest steps are also made lighter where the
measurements show it is worth it.

## Known facts (2026-11-04)

- **Owner's report:** the PC crashed several times during fuzz runs.
  `journalctl --list-boots` shows reboots at 10:41, 10:56, 10:57, 12:53
  and 12:54. T60's session started at 10:37:53 and its log is empty.
- **Machine:** 8 threads, 7.8 GB of RAM, 7.6 GB of zram swap, about
  2.5 GB available with the desktop open. SSD with the `bfq` scheduler,
  so I/O priority works. `/tmp` is a RAM-backed tmpfs (3.9 GB): **never put
  a target dir there**, its files count as memory.
- **Today nothing is limited:** every `cargo` step uses 8 jobs and 8 test
  threads.
- **The cause of the crashes is not proven.** No step below passes 2 GB,
  so the gates alone do not explain them. They add to what is already
  running: `user-1000.slice` peaked at 6.3 GB of memory plus 7.2 GB of swap
  by 10:40, in a session open for 13 h 33 min. The journal shows no kernel
  OOM kill, and `swaync` and `waybar` failing just before an orderly
  reboot. Thermal or GPU causes are not ruled out.

### Measurements at `436f9ed`

Target dir on disk, cgroup limits, no swap, one step at a time. *Peak* is
the cgroup's, which includes reclaimable page cache; *largest* is the
biggest single process. *CPU pressure* is the highest
`/proc/pressure/cpu` `avg10` sampled, which is a proxy for how the desktop
feels, not a measure of it.

| Step | Time | CPU time | Peak | Largest | CPU pressure |
|------|------|----------|------|---------|--------------|
| clippy, cold, 3 jobs | 19 s | 40 s | 645 MB | 558 MB | 0.5 |
| clippy, warm, 3 jobs | 4 s | 4 s | 407 MB | 432 MB | 0 |
| test build, cold, 3 jobs | 1 min 05 s | 2 min 54 s | 1.5 GB | 1.1 GB | 7.3 |
| test build, cold, 8 jobs | 1 min 02 s | 5 min 21 s | 2 GB | 1.1 GB | 18.6 |
| test build, cold, 3 jobs, **1 GB cap** | 1 min 09 s | 2 min 58 s | 1 GB | — | — |
| test build, warm, 3 jobs | 7 s | 10 s | 1 GB | 536 MB | 2.7 |
| test run, 3 threads | 20 s | 33 s | 159 MB | 91 MB | 2.6 |
| test run, 8 threads | 14 s | 59 s | 126 MB | 122 MB | 10.5 |
| doc | 1 s | 1 s | 210 MB | 271 MB | 0.5 |
| fuzz build, cold, 3 jobs | 1 min 08 s | 2 min 32 s | 902 MB | 456 MB | 0.4 |
| fuzz build, cold, 8 jobs | 1 min 04 s | 3 min 40 s | 1.1 GB | 454 MB | 10.1 |
| fuzz build, warm, 3 jobs | 40 s | 58 s | 637 MB | 453 MB | 9.5 |
| fuzz run, 20 000 cases, 3 threads, 1 GB cap | 2 min 59 s | 8 min 04 s | 70 MB | — | — |
| fuzz run, 20 000 cases, 8 threads, no cap | 3 min 25 s | — | — | — | — |

What they show:

- **3 threads cost almost nothing.** Cold builds take about the same time
  with half the CPU time, and the fuzz run was no slower. Only the test
  run loses time (6 s).
- **Everything fits the limits.** Each step passed at 3 jobs. The test
  build, the only step past 1 GB, also passed under a 1 GB cap, with no
  time lost.
- **Thirstiest for memory:** the test build, cold and warm. It links 7
  test binaries, and its 1 GB warm peak compares with 536 MB for the
  largest process.
- **Thirstiest for CPU over time:** the fuzz run, about 2.7 cores for 3
  minutes, at only 70 MB.
- **Worst warm build:** the fuzz build, 40 s of fat LTO for every changed
  line.
- A whole session at the limits takes about 6 min cold and 4 min warm.

## Proposal (settle in the spec step)

**Required, for every `cargo` step in `scripts/check.sh`:**

1. `CARGO_BUILD_JOBS=3` and `RUST_TEST_THREADS=3`.
2. A transient unit per step: `systemd-run --user --scope -q -p
   MemoryMax=1G -p MemorySwapMax=0 -p CPUQuota=300% …`. Use `--scope`: it
   inherits the environment, whereas a service drops `PROPTEST_CASES`
   unless passed with `-E`. `-j3` limits rustc processes, not the threads
   inside each one, so `CPUQuota` is what holds a build to 3 CPUs.
3. Lowest priority, so the browser wins: `nice -n 19` and `ionice -c 3`
   (or `CPUWeight=` and `IOWeight=` on the unit). None of this is measured
   yet.
4. Without a usable `systemd-run --user` (some CI runners), keep what
   still applies and print that the memory cap is not enforced.
5. Keep the limits as named values at the top of `scripts/check.sh`.

**Candidates: measure each, keep only what clearly helps:**

6. A `fuzz` profile (`inherits = "release"`, no fat LTO, more codegen
   units), run with `cargo test --profile fuzz`. Worth it if the warm build
   drops well below 40 s and the 20 000 cases do not get slower by more
   than it saves.
7. `[profile.test] debug = "line-tables-only"` to shrink the test
   binaries and what linking them needs, if backtraces stay useful.
8. Fewer test binaries, by merging the small `tests/*.rs` into one, if the
   link peak justifies the churn.

## Spec

Mechanism: [[ADR-T77-1 Test gates run in capped low-priority scopes]].
The `tests/ci.rs` tests run the real `scripts/check.sh --fuzz` with stand-in
`cargo`, `systemd-run` and `systemctl` first on the `PATH`; `nice` and
`ionice` are the real ones, and the stand-in `cargo` records what it sees.

- **AC-1** — Every `cargo` step in `check.sh` runs with 3 build jobs and 3
  test threads. *Test:* `tests/ci.rs::gates_limit_threads`
- **AC-2** — Every such step runs in a `systemd-run --user --scope` with
  `MemoryMax=1G`, `MemorySwapMax=0` and `CPUQuota=300%` when the probe
  scope starts. *Test:* `tests/ci.rs::gates_run_under_a_memory_cap`
- **AC-3** — Every such step runs at niceness 19, in the idle I/O class,
  and with `CPUWeight=1` and `IOWeight=1` on its scope. *Test:*
  `tests/ci.rs::gates_run_at_low_priority`
- **AC-4** — When the probe scope fails, every gate still runs, with the
  thread limits and the priorities, and a notice says the memory cap is
  not enforced. *Test:* `tests/ci.rs::gates_warn_without_a_memory_cap`
- **AC-5** — Going over the memory limit fails `check.sh` with a message
  naming the step and the limit; an ordinary failure mentions no limit.
  *Tests:* `tests/ci.rs::gates_name_step_and_limit_when_out_of_memory`,
  `tests/ci.rs::gates_blame_no_limit_for_an_ordinary_failure`. *Manual:*
  a real run with `MemoryMax` far too low, with the desktop staying up.
- **AC-6** — The measurements above are repeated at the end, with a clean
  target dir on disk, and recorded in the *Log*. A clean
  `scripts/check.sh --fuzz` runs 20 000 cases, passes, and takes no more
  than about 7 minutes. *Manual.*
- **AC-7** — Each candidate (6–8) is measured and either kept or
  rejected, with the numbers, in the *Log*. *Manual.*
- **AC-8** — With the browser open, the owner runs a full
  `scripts/check.sh --fuzz` while scrolling a page and playing a video:
  no freeze, no crash. *Manual, by the owner.*

The models step is skipped: the task adds no types, only shell and tests.

## Out of scope

- Fewer cases: 20 000 stays.
- Lowering what the desktop, the browser or other sessions use.
- Fixing a crash whose cause turns out not to be the gates (thermal,
  GPU, memory held by long sessions): that would be its own task, with
  the evidence from AC-8.
- `scripts/package.sh` and the shipped `release` profile.
- Clearing the 16 GB `target/`.

## Files owned

- `scripts/check.sh`
- `tests/ci.rs`
- `Cargo.toml` (only for a candidate that is kept)
- `docs/process/Quality Gates.md` (the limits; its *Fuzz* row also lacks
  `--release`)

## Subtasks (one commit each)

- [x] spec — `docs(T77): specify limits for the test gates`
- [x] tests — `test(T77): add failing tests for the gate limits`
- [x] behaviour — `feat(T77): run the gates within fixed limits at low priority`
- [x] behaviour — `feat(T77): keep the candidates that measured better`
- [x] quality — `chore(T77): pass clippy and rustfmt`
- [x] docs — `docs(T77): record the limits and the measurements`

## Learning path

None planned.

## Log

- 2026-11-04 — created from the owner's report (3 threads, 1 GB of RAM),
  then extended by the owner to every step of a test session, so the
  browser stays usable while it runs.
- 2026-11-08 — implemented. Spec step: `CPUQuota` turns the cpu controller
  on, so `nice` alone would not rank the gates against the browser; hence
  `CPUWeight` and `IOWeight` too (ADR-T77-1).
- 2026-11-08 — AC-5, real run: `MEMORY_MAX=100M` in a temporary copy of the
  script gave `check.sh: clippy went over the 100M memory limit`, exit 143,
  no failed unit left behind, desktop up. The `Result=oom-kill` signal held
  in 25 of 25 trials.
- 2026-11-08 — AC-6, measured at the limits (1 GB, no swap, 3 CPUs, lowest
  priority), target dir on disk, one sample each, with the owner's desktop
  open. Same columns as *Known facts*.

  | Step | Time | CPU time | Peak | Largest | CPU pressure |
  |------|------|----------|------|---------|--------------|
  | clippy, cold | 19 s | 39 s | 791 MB | 558 MB | 0 |
  | clippy, warm | 3 s | 4 s | 396 MB | 434 MB | 0 |
  | test build, cold, before line tables | 1 min 04 s | 2 min 40 s | 1 GB | 1056 MB | 5.4 |
  | test build, warm, before line tables | 15 s | 10 s | 1 GB | 520 MB | 3.4 |
  | test build, cold, line tables (kept) | 55 s | 2 min 21 s | 1 GB | 895 MB | 2.3 |
  | test build, warm, line tables (kept) | 4 s | 7 s | 906 MB | 527 MB | 1.9 |
  | test run | 12 s | 28 s | 128 MB | 90 MB | 0 |
  | doc | 1 s | 1 s | 204 MB | 269 MB | 1.8 |
  | fuzz build, cold | 1 min 04 s | 2 min 21 s | 990 MB | 453 MB | 1.4 |
  | fuzz build, warm | 34 s | 54 s | 559 MB | 458 MB | 0 |
  | fuzz run, 20 000 cases | 2 min 48 s | 7 min 35 s | 80 MB | 71 MB | 0.2 |

  A clean `scripts/check.sh --fuzz` with these limits passed in **5 min
  41 s** (budget: about 7 min), 20 000 cases, no kernel OOM kill.
- 2026-11-08 — AC-7, one sample each, so small differences are noise:
  - **6, `fuzz` profile (`lto = "off"`, 16 codegen units): rejected.** The
    warm build drops from 34 s to 13 s and the cold one from 64 s to 48 s,
    but the 20 000 cases go from 168 s to 191 s. It loses 2 s on a warm
    session and 7 s on a cold one.
  - **7, `[profile.test] debug = "line-tables-only"`: kept.** Test
    artifacts 897 MB to 434 MB, largest process 1056 MB to 895 MB, cold
    build 64 s to 55 s. A panic backtrace still shows file and line.
    Debugger variable inspection in test binaries is lost, and `cargo test`
    no longer shares its dependencies with `cargo build`.
  - **8, one merged test binary: rejected.** Measured in a throwaway copy:
    cold build 114 s against 64 s (other load may have added to that),
    largest process 1097 MB against 1056 MB, warm build no better, test
    artifacts 638 MB. Line tables get further with no churn, and it also
    needs a shared `common` module and changes outside *Files owned*.
- 2026-11-08 — AC-8 is left for the owner: run `scripts/check.sh --fuzz`
  with the browser open, scrolling a page and playing a video.
- Feature note and tutorial skipped: this task's `feature` and `tutorial`
  are empty, as it changes tooling only.
- 2026-11-08 — integrated on `main` (merge `00e1cf4`), no conflicts; the owner's AC-8 run with the browser open is still open.
