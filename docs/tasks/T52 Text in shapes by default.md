---
id: T52
title: Text in shapes by default
status: review
wave: 32
branch: task/T52-shape-text-default
depends_on: []
adrs: [ADR-T52-1]
feature: "Text in shapes"
tutorial: "31 Prototyping an interaction behind a switch"
tags: [task]
---

# T52 Text in shapes by default

## Goal

Writing inside rectangles, ellipses (circles) and grid cells is a normal
feature, on without `DRAW_EXPERIMENTAL=shape-text`. The one exception:
a **numbered** rectangle or ellipse (one with a label from the numbering
helper, [[T19 Auto-numbering]]) cannot hold text. Its number is its
content. Grids, including numbered grids ([[T22 Numbering drives grid
indices]]), keep text in their cells.

## Known facts (2026-11-02)

- The feature came from [[T31 Text in rectangles and grid cells]]
  behind the switch of [[ADR-T31-1 Shape text prototype behind a
  switch]]: `core::experimental::{Experimental, SHAPE_TEXT,
  EXPERIMENTAL_ENV}`, read once in `shell::app::run` and carried in
  `Helpers::experimental`.
- Code that checks the switch: `core::editor`, `core::tools::{text,
  select}`, `core::shape`, `shell::render`, `shell::app`, and
  `tests/fuzz.rs` (which fuzzes both settings).
- `Shape::Rect` and `Shape::Ellipse` have `label: Option<u32>` and
  `text: Option<ShapeText>`. Today the text is "drawn centred inside
  *instead of* the label", so a numbered shape with text hides its
  number.
- Feature note: `Text in shapes (experimental)` (now [[Text in shapes]]),
  tutorial step 31.
  The README lists `shape-text` under `DRAW_EXPERIMENTAL`.
- There is no file format, so no saved documents have to keep working.

## Proposal (settle in the spec step)

1. **Promote**: a new ADR supersedes ADR-T31-1 (`supersedes:
   [ADR-T31-1]`, which becomes `superseded`). Text in shapes is always
   on.
2. **Remove the switch**: decide whether `core::experimental` stays as
   an empty, reusable mechanism (`DRAW_EXPERIMENTAL` accepted but
   naming nothing), or goes away. Since 2.0.0 adds no features
   ([[T50 Version 2.0.0 refactor]]), removing it is the likely choice,
   with `DRAW_EXPERIMENTAL=shape-text` ignored silently or with a
   one-line notice for one release.
3. **Numbered shapes refuse text**, covering every way in:
   - opening the text editor on a numbered rect/ellipse (double click,
     typing with it selected, the Text tool clicking inside it) does
     nothing there; the Text tool creates a free text block instead,
     as on empty canvas;
   - pasting or duplicating keeps the rule (a numbered copy has no
     text);
   - numbering a rect/ellipse that already holds text: the spec
     decides between leaving it unnumbered (text wins, the counter
     does not advance) and refusing silently. The rule: text is never
     lost without an undo step.
   - The invariant `label.is_some() ⇒ text.is_none()` for Rect/Ellipse
     is asserted in the fuzz harness after every event.
4. **Docs**: the feature note loses "(experimental)" (rename and link
   updates), the README moves it from the experimental list to the
   features, and the cheat sheet lists how to write in a shape.

## Spec

Decisions: [[ADR-T52-1 Text in shapes always on, numbered shapes hold
no text]] (supersedes [[ADR-T31-1 Shape text prototype behind a
switch]]).

- **AC-1 Always on.** With a plain `Editor::new()` (no setting) the Text
  tool opens the text of a rectangle, an ellipse or a grid cell under a
  click; Select's second click on the only selected rect/ellipse/grid
  opens it, and a press inside the only selected shape keeps it.
  Tests (the T31 tests, now without a switch):
  `editor::tests::shape_text_text_tool_click_in_rect_edits_its_text`,
  `…_in_ellipse_edits_its_text`, `…_in_grid_edits_that_cell`,
  `shape_text_select_second_click_edits_selected_shape`,
  `select::tests::shape_text_press_inside_only_selected_rect_keeps_it`,
  `tests/fuzz.rs::sessions_reach_shape_text`.
- **AC-2 The switch is gone.** `core::experimental` (`Experimental`,
  `SHAPE_TEXT`, `EXPERIMENTAL_ENV`), `Helpers::experimental`,
  `Editor::with_experimental` and `Editor::experimental` are deleted;
  the shell no longer reads `DRAW_EXPERIMENTAL` (setting it does
  nothing, silently). The README has no `DRAW_EXPERIMENTAL` row and no
  *Experimental* section. Tests: the build (no reference compiles),
  `shell::app::tests::draw_experimental_is_not_read_or_documented`.
  The off-switch tests (`shape_text_off_*`,
  `switch_off_sessions_never_create_shape_text`,
  `with_experimental_sets_the_switch_in_helpers`, the
  `experimental::tests`) are deleted with the switch; the fuzz
  properties that ran with the switch on become the only ones.
- **AC-3 Snapshot format unchanged.** The `helpers` record keeps its
  seventh token (the old switch), now reserved: written as `0`, read as
  a bool and ignored; shape text loads whatever it says. The fixtures in
  `tests/snapshots/` are byte-identical. Tests:
  `editor::tests::snapshot_helpers_reserved_token_is_ignored`, the
  `tests/snapshots.rs` fixture tests, fuzz
  `snapshot_round_trip_continues_equal`.
- **AC-4 A numbered rect/ellipse never opens a text edit.** The Text
  tool clicking inside a numbered rectangle or ellipse starts a free
  text block there, as on empty canvas; Select's second click on a
  selected numbered shape opens nothing; typing with it selected adds
  no text to it. `Shape::text_host_at(p)` is `text_slot_at(p)` except
  `None` for a labelled rect/ellipse, and every opening path uses it.
  Tests: `shape::tests::text_host_at_skips_numbered_shapes`,
  `editor::tests::shape_text_text_tool_in_numbered_rect_starts_free_block`,
  `shape_text_select_second_click_on_numbered_ellipse_opens_nothing`,
  `shape_text_typing_with_numbered_rect_selected_adds_no_text`,
  `select::tests::press_inside_only_selected_numbered_rect_clears_selection`.
- **AC-5 Numbering never covers text: text wins.** `Shape::with_label`
  leaves a rect/ellipse that holds text unchanged, and
  `Shape::with_slot_text` never puts text into a labelled one; so
  `numbering::label_new` leaves a shape with text unnumbered and the
  counter does not advance. No text is lost. Tests:
  `shape::tests::with_label_keeps_a_shape_with_text_unnumbered`,
  `shape::tests::with_slot_text_never_fills_a_numbered_shape`,
  `numbering::tests::label_new_leaves_shape_with_text_unnumbered`.
- **AC-6 Paste and duplicate keep the rule.** Copies of a numbered
  shape have its label and no text; copies of a shape with text keep
  the text and no label. Test:
  `editor::tests::shape_text_paste_and_duplicate_keep_numbered_shapes_text_free`.
- **AC-7 Fuzzed invariant.** After every event the fuzz harness checks
  `label.is_some() ⇒ text.is_none()` for every Rect/Ellipse in the
  document (`check_shape_text`), and sessions that number the shape
  before typing into it reach free text blocks but never numbered shape
  text. Tests: `tests/fuzz.rs::editor_never_panics_and_keeps_invariants`,
  `numbered_host_sessions_never_fill_numbered_shapes`.
- **AC-8 Snapshots keep the invariant.** Loading a snapshot whose
  document holds a labelled rect/ellipse with text fails with a line
  number. Test:
  `editor::tests::read_snapshot_rejects_numbered_shape_with_text`.
- **AC-9 Grids keep their text.** Cells of a numbered grid (`N` shows
  indices) still hold text. Test:
  `editor::tests::shape_text_numbered_grid_cells_hold_text`.
- **AC-10 Rendering.** A numbered shape always shows its number
  (`render::shown_label`). Test:
  `render::tests::numbered_shape_shows_its_label`.
- **Docs.** The feature note is renamed to [[Text in shapes]]; the
  README moves it to *Features* and the keyboard reference; the
  [[Contest cheat sheet]] says how to write in a shape; tutorial 31 gets
  a "how the prototype graduated" addendum.

## Out of scope

- New text features (alignment, wrapping changes, text on lines).
- Text in numbered grids' index labels.

## Files owned

- `src/core/experimental.rs`, `src/core/mod.rs`, `src/core/editor.rs`,
  `src/core/shape.rs`, `src/core/tools/text.rs`,
  `src/core/tools/select.rs`, `src/core/numbering.rs`
- `src/shell/app.rs`, `src/shell/render.rs`
- `tests/fuzz.rs`
- `README.md`, `docs/features/Text in shapes (experimental).md`
  (renamed), notes linking to it
- new ADR `ADR-T52-*`

## Subtasks (one commit each)

- [x] spec — `docs(T52): …`
- [x] tests — `test(T52): …`
- [x] models — `feat(T52): …`
- [x] behaviour — `feat(T52): …`
- [x] quality — `refactor(T52): …`
- [x] docs — `docs(T52): …`

## Learning path

Extends step 31 (*Prototyping an interaction behind a switch*) with how
a prototype graduates: removing a switch and keeping an invariant.

## Log

- 2026-11-02 — note written at the owner's request; ships in 1.5.0
  ([[T53 Release 1.5.0]]).
- 2026-11-04 — Implemented on `task/T52-shape-text-default` (unattended
  batch). Every AC-1..AC-10 test passes; `scripts/check.sh` and
  `scripts/check.sh --fuzz` green. ADR-T52-1 added; ADR-T31-1 set to
  `superseded` (`superseded_by: ADR-T52-1`), body untouched.
- 2026-11-04 — UNATTENDED CHOICE: the snapshot `helpers` record keeps
  its seventh token as a reserved value (written `0`, ignored on read)
  instead of dropping it, because dropping it changes every fixture in
  `tests/snapshots/` (not owned). `core::snapshot`'s `shape_text` reader
  flag (not owned) stays; the editor always turns it on. A later
  snapshot-format task may remove both.
- 2026-11-04 — UNATTENDED CHOICE: the invariant lives in `Shape`
  (`with_slot_text` / `with_label` guards, `text_host_at`) rather than
  in each tool; `text_slot_at` is unchanged so the eraser's fill clearing
  on numbered shapes keeps working. A press inside the only selected
  *numbered* unfilled rect clears the selection (as before T31), since
  it can host no text.
- 2026-11-04 — Deviation: the T31 fixture `tools::text::tests::host_rect`
  had `label: Some(3)`, a state the invariant now forbids; the label was
  set to `None` in the behaviour step (the tests don't depend on it).
  `render::tests::label_is_hidden_by_shape_text` became
  `numbered_shape_shows_its_label`.
- 2026-11-04 — Files beyond the list: `docs/features/Contest cheat
  sheet.md` (per the plan), `docs/architecture/Architecture.md` (module
  table and invariants), `docs/tasks/T31 …` and tutorial 31 (links,
  addendum), `src/core/tools/text.rs` test fixture above.
- 2026-11-04 — For the integrator: `docs/features/Feature Index.md`
  links `[[Text in shapes (experimental)]]`; change it to
  `[[Text in shapes]]`. `Decision Log` needs ADR-T52-1 and ADR-T31-1 →
  superseded. `CHANGELOG.md`: text in shapes on by default,
  `DRAW_EXPERIMENTAL` removed. The Task Board row may mention the old
  name too.
- 2026-11-04 — Stash `batch-T52-2026-11-04 pre-rename copy of Text in
  shapes note` holds a copy of the new feature note made before the
  `git mv` (the quoted form was refused); its content is committed, so
  the stash can be dropped.
- 2026-11-04 — Manual checks for the owner (need the running app):
  without any env var, type in a rectangle, an ellipse and a grid cell;
  number a rect, then try to type in it (a free text block appears);
  `render::shown_label` still hides a label under text as a guard,
  unreachable now.
