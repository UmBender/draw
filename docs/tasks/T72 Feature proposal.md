---
id: T72
title: Feature proposal
status: review
wave: 33
branch: docs/T72-feature-proposal
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, docs, proposal]
---

# T72 Feature proposal

## Goal

A written proposal of features draw could add, so the owner can choose
what to build. It is a decision document, not code. Each idea has: the
problem it solves for a competitive programmer, what it looks like in
use (keys, gestures), a size estimate (S/M/L), what it would cost in
start-up time, frame time and binary size, and its risk to the
*minimal, fast, keyboard-first* goal. The owner marks each idea
*accepted*, *later* or *rejected*. Accepted ideas become task notes in a
follow-up.

## Known facts (2026-11-03)

- Owner (2026-11-02): 1.5.0 ([[T53 Release 1.5.0]]) is the last feature
  release, 2.0.0 ([[T50 Version 2.0.0 refactor]]) adds no features, and
  only bug fixes follow. So an accepted idea must fit into 1.5.0, or the
  owner changes that decision. The proposal says this up front.
- What draw has: [[Feature Index]]. What it does not have: the *Limits*
  sections of the feature notes (e.g. [[TeX formulas#Limits]]) and
  `README.md` *Limitations*.
- There is no file format: documents live only in memory.

## Proposal (starting list; the task refines it)

Grouped by area:

1. **Keeping work**: save/load a document (a small text format),
   export to PNG/SVG, copy the selection as an image to the clipboard,
   autosave and crash recovery.
2. **Contest drawing**: graph and tree templates (n nodes in a circle
   or a layered tree, edges by typing `1-2 2-3`), an array/string
   strip with index labels, a 2D grid filled from pasted input, an
   interval/segment-tree drawing, a coordinate plane with axes.
3. **Formulas**: display mode `$$…$$`, `\left`/`\right` growing brackets,
   matrices (`pmatrix`, `cases`), extensible arrows (`\xrightarrow{f}`),
   commutative diagrams, user macros (`\newcommand`).
4. **Canvas**: layers or frames (pages), a minimap, a laser pointer
   that fades, presenter mode with a fixed zoom, a ruler and a protractor.
5. **Editing**: align and distribute, group/ungroup, lock, z-order
   keys, find text, a command palette.
6. **Input**: pressure-sensitive highlighter, shape recognition
   (rough circle to an ellipse), handwriting-to-TeX (likely rejected:
   size and dependencies).

## Spec

- **AC-1** — `docs/proposals/Feature proposal.md` exists (from a new
  template in `docs/templates/Proposal.md`) and every idea has the
  fields listed in *Goal*.
  *Check:* every `### ` idea heading is followed by the labels
  **Problem**, **In use**, **Size**, **Cost**, **Risk**.
- **AC-2** — Each idea links the feature note it extends or the limit it
  removes, and says whether it needs a new dependency (ADR-0014) or
  breaks a budget (ADR-T14-1, ADR-T24-5).
  *Check:* every idea has **Extends** with a `[[wikilink]]` that resolves
  to a note, **Dependency** and **Budgets**.
- **AC-3** — The ideas are ranked (value ÷ cost), with a recommended
  short list of at most five for 1.5.0.
  *Check:* the ranking table has one row per idea, sorted by score, each
  score equal to value ÷ cost; the short list has at most five entries,
  all of them in the table.

This task produces a document, so the workflow's *Tests*, *Models*,
*Behaviour* and *Quality* steps have no code to act on and are skipped.
The checks above replace the tests. They are run from the scratch area
and reported in the *Log*; `tests/` is not in *Files owned*.

Scoring, so the ranking can be re-derived: **value** 1–5 for a
competitive programmer during a contest; **cost** = size (S 1, M 2,
L 4) + risk to the goal (low 0, medium 1, high 2).

## Out of scope

- Implementing anything, or writing task notes for accepted ideas
  (follow-up, after the owner decides).

## Files owned

- `docs/proposals/Feature proposal.md` (new folder), `docs/templates/Proposal.md`
- this note

## Subtasks (one commit each)

- [x] research — read every feature note's *Limits*, list the ideas
- [x] proposal — `docs(T72): propose features for draw`
- [ ] decision — owner's verdicts recorded in the proposal, `status: done`

## Learning path

None: a docs-only task.

## Log

- 2026-11-03 — note written at the owner's request.
- 2026-11-09 — [[Feature proposal]] written: 35 ideas, ranked, with a
  short list of five. The starting list was refined: the array strip
  became part of C1, the graph and tree templates became C2, and six
  ideas came from the *Limits* sections (E1–E4, F5, K1).
- 2026-11-09 — AC-1 to AC-3 checked with a script over the proposal
  (fields, links resolve, table matches the ideas, sorted, scores
  re-derived, short list at most five): all pass. Binary measured at
  1 966 008 bytes against the 2 MiB limit in `scripts/package.sh`; the
  per-idea binary and frame-time costs are estimates.
- Waiting for the owner's verdicts in the proposal's *Ranking* table.
  The *decision* subtask and `status: done` follow them.
