---
id: ADR-T29-2
title: Text edit mode
status: accepted
kind: decision
date: 2026-10-14
task: T29
builds_on: [ADR-0005, ADR-T08-1, ADR-T11-1, ADR-T24-1, ADR-T29-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T29-2 Text edit mode

## Context

The app is keyboard-first: almost every printable key is a shortcut
([[ADR-T11-1 Exact modifier matching]]). Typing into a text block must
never fire one, leaving text mode must be one key, and the edit must fit
the transaction-log undo ([[ADR-0005 Undo via transaction log]]) and the
cached document layer ([[ADR-T24-1 Document layer keyed by a document revision]]).
`core` has no clock, so it cannot tell a double-click from two clicks.

## Decision

- **Typed characters.** `InputEvent::Char { ch, mods }` carries what the
  system's keyboard layout produced (miniquad's `char_event`; dead keys
  are composed by the system). It is ignored outside edit mode. In edit
  mode a `char` is inserted unless it is a control character or `Ctrl` is
  held. IME composition is out of scope.
- **Edit session.** `tools::text::State` holds at most one `TextEdit`: the
  block's origin, text, size, style, the caret (a byte index, always on a
  `char` boundary) and, when re-editing, the id of the original block. The
  session outlives pointer gestures; it is not part of the document until
  it ends.
- **Keys in edit mode** never reach the keymap:

  | Key | Action |
  |-----|--------|
  | `Esc`, `Ctrl+Enter` | commit and leave |
  | `Enter` | new line |
  | `Backspace` / `Delete` | delete the `char` before / after the caret |
  | `←` `→` | previous / next `char` |
  | `↑` `↓` | previous / next line, same column (clamped) |
  | `Home` / `End` | start / end of the line |
  | `Tab` | insert `TAB_SPACES` (4) spaces |
  | `Ctrl+Z` | cancel the edit |
  | anything else | nothing (its `Char` event does the typing) |

  `Space` does not arm space-panning in edit mode.
- **Commit** happens on `Esc`, `Ctrl+Enter`, a left or right press outside
  the block that does not pan (the press then goes on to the active tool,
  so with the Text tool it starts the next block at once), and before any
  command applied from the toolbar. Panning and zooming keep the edit
  open. A new block becomes one `Insert`; a changed block one
  `Replace`; a block edited to blank text (empty or only whitespace) one
  `Remove`. A new blank block and an unchanged block leave no history
  entry. So **one history entry per finished edit**.
- **Cancel.** `Ctrl+Z` (or Undo from the toolbar) while editing throws the
  edit away: a new block vanishes, a re-edited block keeps its old text.
  Undo inside the block's typing is not offered — the edit is short and
  `Ctrl+Z` after committing undoes it as a whole.
- **Opening.** With the Text tool a press on a text block re-opens it with
  the caret at the press; elsewhere it starts a new block whose top-left
  corner is the press point, snapped like a shape's start point (grid →
  align). With Select, a click on a text block that was **already the only
  selected shape** re-opens it. A double-click is exactly that (the first
  click selects), and no clock is needed; a slower second click does the
  same. A press inside the block being edited only moves the caret.
- **Drawing.** While editing, `Editor::overlay` shows the edited block
  among its shapes and hides the original (so the document layer is
  re-rendered once when the edit starts and once when it ends; each
  keystroke only redraws the overlay). The caret is a line of the text
  colour, `Editor::text_caret` in world space. It **does not blink**: a
  blinking caret would wake the blocking loop twice a second
  ([[ADR-T12-1 Blocking event loop with cached frame]]).

## Alternatives considered

- **Shell-timed double-click event** — a clock in `input_map` and a new
  event; less testable, and two quick clicks on two different blocks need
  extra rules.
- **One history entry per keystroke** — floods the undo stack.
- **In-block undo stack** — more state for little gain in short notes.
- **Insert `\t`** — tab stops would break the one-cell-per-`char` layout.
- **Clicking outside only commits** — every new block would take two
  clicks.

## Consequences

- Edit mode is a modal state the fuzzer must reach; it generates `Char`
  events and Text-tool sessions.
- Selection with a single click on an already selected text block now
  edits it; dragging it still moves it.

## Rollback plan

Remove the edit session and `InputEvent::Char` with `revert(T29)` commits
and a rollback ADR; text blocks would then be read-only.
