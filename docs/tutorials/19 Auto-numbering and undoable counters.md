---
title: Auto-numbering and undoable counters
step: 19
requires: ["[[06 Undo and redo with a transaction log]]", "[[16 Growing a data model without breaking it]]"]
feature: "[[Numbered nodes]]"
code: ["src/core/numbering.rs", "src/core/tools/shape_tool.rs", "src/core/editor.rs"]
tags: [tutorial]
---

# Auto-numbering and undoable counters

> **Learning path step 19.** Requires: steps 6 and 16 · Next: step 20

## Why this matters here

Graph and tree problems are drawn as numbered nodes. Typing numbers is slow
and the app has no text tool, so new nodes number themselves. The hard part
is not the label — it is keeping the counter honest when the user undoes.

## The concept

A counter that lives **outside** the document is state that undo does not
know about. Draw nodes 1 and 2, undo the 2: the document is back, but a
naive counter says the next node is 3. There are three classic fixes:

1. **Store the counter in the history** — each transaction remembers the
   counter before it; undo restores it. Exact, but the history must carry
   side data.
2. **Derive the counter from the document** — e.g. `max label + 1`. No
   extra state, but then "restart at 1" is impossible while labels exist.
3. **Derive the change, not the value** — look at what a step did to the
   document and move the counter accordingly.

draw uses the third. Before and after each step, count the shapes that
carry the interesting number:

```
gesture end / redo : count(next)     grew   → next + 1
undo               : count(next − 1) shrank → next − 1
```

Counting (not "is it present?") matters after a restart: with an old `1`
in the document, a new `1` raises its count from 1 to 2, so the counter
still moves, and undoing it drops the count back, so it moves back.

## How draw implements it

- `numbering::label_new` — `Some(next_number)` on `Rect`/`Ellipse` while
  numbering is on; everything else passes through. Labels exist only on
  those two variants, so the type system already forbids a numbered line.
- `shape_tool::on_pointer` and `shape_tool::preview` call `label_new` with
  the helpers from `ToolCtx::style` / `ToolView::style`, so the preview
  shows the number the commit will use.
- `Editor::pointer_up` counts `next_number` before handing `Up` to the tool
  and calls `advance_numbering` after it; `Command::Redo` does the same.
  `Command::Undo` counts `next_number − 1` around `History::undo` and calls
  `numbering::roll_back`, which never goes below `FIRST_NUMBER`.
- Why not option 1? The shape tool commits through `ToolCtx::commit`
  straight into `History`; the editor never sees the transaction, and
  `History` drops old entries at its limit. A parallel stack would drift.
  The trade-off is written down in [[ADR-T19-1 Numbering counter and undo]].

## Try it

- Add a test in `numbering.rs`: `advance(5, 3, 3)` must stay 5. Why is
  "count unchanged" the common case for a pen stroke?
- Change `advance` to use presence (`before == 0 && after > 0`) and run
  `cargo test editor::tests::reset_restarts_at_one` — see it fail, and
  explain why.
- Run `PROPTEST_CASES=20000 cargo test --test fuzz`: random keys press `N`
  and `Shift+N`, and the invariants check every label and the counter stay
  `≥ 1`.

## Further reading

- [[06 Undo and redo with a transaction log]] — what a transaction is.
- Command pattern and memento pattern (Gamma et al.) — option 1 is a memento
  attached to each command.
