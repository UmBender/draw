---
id: ADR-T82-1
title: Shipped binary limit of 20 MiB
status: accepted
kind: decision
date: 2026-11-09
task: T82
builds_on: [ADR-T27-3]
amends: [ADR-T29-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T82-1 Shipped binary limit of 20 MiB

## Context

[[ADR-T29-1 Text shape with monospaced layout and an embedded font]] states
that the release binary must stay under 2 MB, and `check_debug_symbols` in
`scripts/package.sh` enforces it as `max_binary_bytes`, counted in MiB. On
2026-11-09 the stripped binary was 1 966 008 bytes, about 128 KiB under the
limit. Several features in the [[Feature proposal]] would not fit in that
space, and the owner raised the limit on 2026-11-09 so that features are no
longer held back by size.

## Decision

- `max_binary_bytes` in `scripts/package.sh` becomes
  `$((20 * 1024 * 1024))`: the shipped, stripped binary must stay under
  20 MiB (20 971 520 bytes).
- The owner said 20 MB; the limit counts in MiB, as it did for 2 MiB.
- This amends only the 2 MB figure of ADR-T29-1.
- Two things still bound the size:
  - **Start-up time.** The binary is read and mapped at every launch, and
    the window must keep opening fast. A larger binary is acceptable only
    while it does.
  - **The per-font guards.** Each embedded font keeps its own limit, such
    as the math font's in
    [[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]].
    They stay as they are.

## Alternatives considered

- **Keep 2 MiB** — about 128 KiB is left, so the next features would each
  have to win space back first.
- **No limit** — an accidental jump, such as an unstripped binary or a full
  font embedded by mistake, would pass unnoticed.
- **Exactly 20 000 000 bytes** — the script has always counted in MiB, and
  the difference has no practical meaning.

## Consequences

- Features are no longer held back by the binary size.
- The packaging check catches only large jumps now. A slow growth of a few
  hundred KiB goes unnoticed until start-up time suffers.
- `tests/packaging.rs` pins the value, so changing it needs a new ADR.

## Rollback plan

A rollback ADR plus a `revert(T82)` commit restores `2 * 1024 * 1024` and
the test. That needs the binary to be under 2 MiB again.
