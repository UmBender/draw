---
id: ADR-T74-1
title: Line-based editor snapshot format
status: accepted
kind: decision
date: 2026-11-03
task: T74
builds_on: ["[[ADR-0003 Headless core and thin shell]]", "[[ADR-0014 Minimal dependencies and no unsafe]]", "[[ADR-0009 Fuzzing with proptest]]", "[[ADR-T49-1 Undo history capped by bytes]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T74-1 Line-based editor snapshot format

## Context

Tests rebuild editor states by replaying thousands of events, and some
states (mid-gesture, a tap-filter phase, an open text block) are awkward
to reach. [[T74 Editor snapshots for tests]] saves the whole headless
`Editor` to a text file and loads it back. No `serde` (ADR-0014), so the
format is written and parsed by hand in `core`, which is fuzzed.

## Decision

**Format.** UTF-8 text, one record per line, tokens separated by one
space. The first line is `draw-snapshot <VERSION>`; `snapshot::VERSION`
is `1`. Then sections, each opened by a `[name]` line, in a fixed
order: `[editor]`, `[document]`, `[history]`, `[clipboard]`, `[tools]`.
Inside a section every record starts with a fixed key and records come
in a fixed order, so a state has exactly one text (*canonical*):
`to_text(parse(t)) == t` for any text the writer produced.

- Floats are written with `{:?}` (shortest text that parses back to the
  same `f32` bits) and must be finite.
- Optional values are `-` when absent.
- Strings are quoted, with `\\`, `\"`, `\n`, `\r`, `\t` and `\u{hex}`
  for other control characters.
- Shapes are one self-delimiting token run (kind, style, geometry, then
  counted lists), so a `replace` edit holds two shapes on one line.
- Unknown versions, sections or keys are errors that name the line.

**What is saved.** Document (ids and `next_id`), undo and redo stacks
with their limit and byte budget, camera, selection, clipboard, every
tool's state (including a live pen smoother, a shape or grid drag, an
open text edit, an eraser drag, a select marquee or move), active tool,
style and helpers, smoothing, toolbar, viewport, cursor, `Space`, the
gesture, the re-edit press, the tap filter (slop and phase) and the
pressure.

**What is not saved**, and why:

- The keymap is a `fn` pointer; a loaded editor uses `keymap::resolve`.
- The document revisions (ADR-T24-1) start at `0`: loading is a fresh
  start, so the shell renders everything.
- `History::changes` restarts at `0` (only its deltas are used), and
  `History::bytes` is recomputed from the loaded stacks: it is an
  estimate from `Vec` capacities, which a load does not reproduce.

**API.** `Editor::snapshot(&self) -> Snapshot`, `Snapshot::to_text`,
`Snapshot::parse(&str) -> Result<Snapshot, SnapshotError>` (a 1-based
line and a message), `Editor::from_snapshot(Snapshot) -> Editor`. A
`Snapshot` is a validated state, so loading cannot fail: every check is
in `parse`. Private tool states encode themselves through
`snapshot::{Writer, Reader}` in their own modules, so no field becomes
public.

**Validation in `parse`.** Floats finite; colours in the palette; grid
sizes and cells in range; stroke width factors one per point; ids
unique and below `next_id`; selection, gesture and edit ids in the
document; text carets on character boundaries; zoom in range; tool,
gesture and tap phases consistent enough that the next event cannot
panic. The parser never panics on any input (fuzzed).

**Equality.** `Editor: PartialEq` compares the saved state, so it
ignores the keymap, revisions, `History::changes` and `History::bytes`.

**Versioning.** Any change to the text is a new `VERSION` with a new
ADR. Old versions are not read: fixtures are regenerated with
`DRAW_UPDATE_SNAPSHOTS=1`.

## Alternatives considered

- **A recorded event script instead of state** — replay is what makes
  tests slow today, and it cannot start mid-gesture.
- **Key–value lines in any order** — easier to hand-edit, but two texts
  per state break golden comparisons and need a sort on write.
- **Binary format** — smaller, but unreadable in diffs and reviews.
- **A separate `Snapshot` data model mirroring every type** — doubles
  the types; wrapping a validated `Editor` keeps one source of truth.

## Consequences

- Every new field in `Editor` or a tool state must be added to the
  format (and `VERSION` bumped), or a round-trip test fails.
- Byte-budget eviction after a load may differ slightly from the
  original session, because `History::bytes` is an estimate.
- Snapshots are for tests and bug reports. A user-facing document file
  ([[T72 Feature proposal]]) would be a separate, smaller format.

## Rollback plan

Remove `core::snapshot`, the per-module encoders, the shell key and env
variable, and the fixtures; tests go back to replaying events.
