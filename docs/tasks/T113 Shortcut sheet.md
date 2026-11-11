---
id: T113
title: Shortcut sheet
status: review
wave: 60
branch: task/T113-shortcut-sheet
depends_on: [T111]
adrs: ["[[ADR-T113-1 Shortcut sheet as a second modal overlay]]"]
feature: "[[Shortcuts and macros]]"
tutorial: ""
tags: [task]
---

# T113 Shortcut sheet

## Goal

`?` opens an in-app cheat sheet of the main shortcuts over the canvas, one topic per page (Tools, Style, Edit, View, Helpers, Gestures, Typing, …). `←`/`→` flip pages, a digit jumps to a topic, and `Esc` closes it the same way it leaves a text block or clears a selection.

## Known facts (2026-11-10)

- `?` (`Shift+/`) is unbound.
- [[Contest cheat sheet]] already groups the shortcuts by topic (Tools, Style, Edit, View, Helpers, Gestures, Typing in a text block, Typing in a shape); every key binding has a description in the keymap table (`src/core/keymap.rs`), and a test checks the cheat sheet against it.
- T111 adds an overlay prompt over the canvas that keys go to while it is open; this task reuses that overlay plumbing, hence the dependency.

## Spec

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — `?` opens the sheet on its first page; `?` again or `Esc` closes it, as `Esc` leaves a text block or clears a selection. In text edit mode `?` types a `?`.
  Tests: `sheet_question_mark_opens_on_the_first_page`, `sheet_question_mark_and_escape_close`, `sheet_question_mark_types_in_a_text_block`, `sheet_does_not_open_over_the_palette`.
- **AC-2** (owner) — One topic per page, in the order of [[Contest cheat sheet]]; the topic names are listed at the top with the current one highlighted. `←`/`→` go to the previous/next page (wrapping), `1`–`9` jump to that topic.
  Tests: `topics_follow_the_cheat_sheet_order`, `sheet_arrows_wrap_pages`, `sheet_digits_jump_to_a_topic`, `sheet_digit_without_topic_is_ignored`.
- **AC-3** (owner) — Each page shows the main shortcuts of its topic as key and description, fitting one window page without scrolling at the default window size; a topic too long for one page is split into "Edit 1/2", "Edit 2/2".
  Tests: `no_page_has_more_rows_than_fit`, `long_topic_is_split_into_numbered_parts`, `split_parts_keep_every_row_once`.
- **AC-4** — The key rows come from the keymap table (each binding gets a topic), so the sheet cannot drift from the real keys; a test checks that every binding appears on exactly one page. Gestures and typing rows that are not key bindings come from one small table in the same module.
  Tests: `every_binding_is_on_exactly_one_page`, `topic_of_matches_the_sections_of_the_keymap`, `gesture_and_typing_rows_have_text`.
- **AC-5** — While open, keys and clicks never reach the tools; nothing is recorded in the history; it is drawn as an overlay only and costs nothing while closed.
  Tests: `sheet_keys_and_pointer_do_not_reach_the_tools`, `sheet_ignores_commands_while_open`, `sheet_open_and_close_record_nothing`, fuzz `sheet_sessions_never_panic_and_change_nothing`.

Decisions beyond the owner's: [[ADR-T113-1 Shortcut sheet as a second modal overlay]]. `UNATTENDED CHOICE:` `src/core/mod.rs`, `src/shell/mod.rs` and a new `src/shell/sheet.rs` are added to the files owned.

## Out of scope

- Searching (that is T111's palette), editing or remapping keys, printing.

## Files owned

- a new `src/core/sheet.rs`, `src/core/mod.rs`, `src/shell/mod.rs`, a new `src/shell/sheet.rs`, `src/core/keymap.rs` (topic per binding), `src/core/editor.rs`, `src/core/command.rs`
- `src/shell/` (overlay drawing)
- `docs/features/Shortcuts and macros.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T113-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T113): specify the shortcut sheet`
- [x] tests — `test(T113): add failing tests for the shortcut sheet`
- [x] models — `feat(T113): add topics to the keymap and the sheet pages`
- [x] behaviour — `feat(T113): show the shortcut sheet on ?`
- [x] quality — `chore(T113): pass clippy and rustfmt`
- [x] docs — `docs(T113): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner; like T111 but a paged cheat sheet. Keys chosen by the owner: `?` to open, `←`/`→` and digits for pages, `Esc` to close.
- 2026-11-11 — implemented on top of T111; `scripts/check.sh` green. The topic of a binding is an exhaustive `match` (`keymap::topic_of`), not a table field ([[ADR-T113-1 Shortcut sheet as a second modal overlay]]). `UNATTENDED CHOICE:` the palette chord and `?` rows sit on the View page (they are not bindings); `ROWS_PER_PAGE` is 14. Manual check for the owner: open the sheet with `?`, flip every page and check none overflows at 1280×800; the topic names wrap to a second line when the window is narrow.
