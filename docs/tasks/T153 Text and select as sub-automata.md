---
id: T153
title: Text and select as sub-automata
status: in-progress
wave: 69
branch: task/T153-text-and-select-as-sub-automata
depends_on: [T136, T138]
adrs: [ADR-T153-1]
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T153 Text and select as sub-automata

## Goal

The text tool and the select tool keep their sub-states in typed enums in `machine/`, so a state that the code does not mean cannot be built and a half-finished edit cannot meet a command that does not expect it.

## Known facts (2026-11-12)

- `core/tools/text.rs` keeps `target: Option<ShapeId>`, `host: Option<Host>`, `resized: bool` and `edit: Option<TextEdit>`; the combinations the code reaches are not written down.
- `core/tools/select.rs` has a private `enum Gesture` (marquee, move, resize, with `additive`, `duplicate`, `proportional`).
- Pen, line, arrow, rectangle, ellipse, grid, bucket and eraser are press, drag, release; `Stage` covers them ([[ADR-T136-1 One gesture variant per tool in the machine]]).

## Spec

See [[ADR-T153-1 Text and select are sub-automata of the machine]].

### Step 1 — the states the code reaches (2026-11-12)

**Text.** `TextEdit` has three constructors, and each fixes `target` and
`host` together: `TextEdit::new` (target `None`, host `None`),
`TextEdit::open_in` (target `None`, host `Some`) and `TextEdit::open`
(target `Some`, host `None`). No code sets either field afterwards, and both
are private to `text.rs`. So of the four `(target, host)` combinations
three are reached and one is not:

| `target` | `host` | Reached by | Variant of `Anchor` |
|----------|--------|------------|---------------------|
| `None` | `None` | `new`: a press on empty space | `Anchor::New` |
| `Some(id)` | `None` | `open`: a press on a text block | `Anchor::Block(id)` |
| `None` | `Some(h)` | `open_in`: a press in a rectangle, ellipse or grid cell | `Anchor::Slot(h)` |
| `Some` | `Some` | nothing | no variant: cannot be built |

The only way to build the fourth is the snapshot reader, which reads the two
columns independently. The writer never writes it, so no snapshot written by
any version holds it.

`resized` is set by `step_size` in all three variants and read by the
editor on commit; it is data of the edit, not a state (all 3 × 2 values are
reached), so it stays a field.

**Decision on the snapshot.** The format does not change (version stays 3):
the `text` record keeps its `<target> <host>` columns. The reader maps them to
an `Anchor`; both set is a `SnapshotError` ("text block has both a target and
a host"). Old snapshots load because the writer never produced that record.

**Select.** `State { gesture: Option<Gesture> }` with a private `Gesture`
of three variants. The states are `Idle`, `Marquee`, `Move`, `Resize`; the
`Option` and the enum merge into one `Select` enum, with `ResizeFrame`,
in `machine/select.rs`. Transitions: `press` (from `Idle`, decided by the hit
test in the tool), `drag` (marquee and move follow the pointer; resize also
`proportional`), `release` (consumes the state) and `cancel`. Each is a
`match` on all four variants with no wildcard.

### Acceptance criteria

- **AC-1** — `machine::text::Anchor { New, Block, Slot }`; `TextEdit` holds one
  `Anchor`, no `target` or `host` field. *Tests:*
  `anchor_new_has_no_target_or_host`, `anchor_block_has_target_only`,
  `anchor_slot_has_host_only`, `edit_open_builds_block_anchor`,
  `edit_open_in_builds_slot_anchor`,
  `edit_new_builds_new_anchor`, `anchor_from_parts_rejects_target_with_host`
  (one per variant and per way in), plus the existing text tests unchanged.
- **AC-2** — `machine::select::Select { Idle, Marquee, Move, Resize }`;
  transitions are exhaustive matches. *Tests:*
  `select_idle_drag_stays_idle`, `select_marquee_drag_follows_pointer`,
  `select_move_drag_follows_pointer`,
  `select_resize_drag_updates_proportional`, `select_cancel_from_each_state_is_idle`
  (returns whether a state was left), plus the existing select tests unchanged.
- **AC-3** — the snapshot format is unchanged. *Tests:*
  `snapshot_roundtrip_text_new`, `snapshot_roundtrip_text_block`,
  `snapshot_roundtrip_text_slot`, `snapshot_roundtrip_select_marquee`,
  `snapshot_roundtrip_select_move`, `snapshot_roundtrip_select_resize`,
  `snapshot_rejects_text_with_target_and_host`, and the existing fixtures
  `tests/snapshots/text-mid-edit.snap` and `select-mid-drag.snap` still load.
- **AC-4** — `tests/model.rs`: `machine_matches_the_model` gets actions that
  open a text block, type and commit, and a select marquee; the model counts
  text blocks. `no_state_traps_the_user` gets the text and select presses in
  its alphabet.
- **AC-5** — no `dyn`, no wildcard arm in `machine/text.rs` and
  `machine/select.rs`. *Tests:* `tests/no_dyn.rs`, `tests/perf.rs` unchanged;
  `machine_sub_automata_have_no_wildcard_arm` greps the two files.

## Out of scope

- The other tools; new user-facing features.
- Anything in another task of [[T50 Version 2.0.0 refactor]].

## Files owned

- `crates/draw-core/src/machine/**`, `crates/draw-core/src/core/tools/text.rs`, `crates/draw-core/src/core/tools/select.rs`, `crates/draw-core/src/core/editor.rs`, `crates/draw-core/src/core/snapshot.rs`
- `tests/model.rs`, `tests/fuzz.rs`
- this note, `ADR-T153-1`

## Subtasks (one commit each)

- [x] spec — `docs(T153): map the text and select states`
- [ ] tests — `test(T153): add failing tests for the states`
- [ ] behaviour — `refactor(T153): move text and select into the machine`
- [ ] quality — `chore(T153): pass clippy and rustfmt`
- [ ] docs — `docs(T153): update the architecture note`

## Log

- 2026-11-12 — created after the owner reviewed ADR-T136-1 and chose to rewrite the sub-states of text and select, not all tools.
