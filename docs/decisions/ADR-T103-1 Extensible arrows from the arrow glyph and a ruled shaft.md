---
id: ADR-T103-1
title: Extensible arrows from the arrow glyph and a ruled shaft
status: proposed
kind: decision
date: 2026-11-10
task: T103
builds_on: ["[[ADR-T30-1 TeX subset, box layout and metrics]]", "[[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T103-1 Extensible arrows from the arrow glyph and a ruled shaft

## Context

T103 adds amsmath's `\xrightarrow`, `\xleftarrow`, `\xleftrightarrow`,
`\xRightarrow`, `\xLeftarrow`, `\xLeftrightarrow`, `\xmapsto` and
`\xhookrightarrow`: an arrow that grows to fit a label above it and an
optional `[below]` label under it. The embedded math font subset has no
extensible arrow pieces, and the renderer cannot clip a glyph. It can
draw glyphs and straight rules (`Ink::Rule`, as fraction bars and root
signs do).

## Decision

- **Node.** A new node `Node::XArrow { name, below, above }` keeps the
  command name, the optional `[…]` label (`Bracket`: its lead whitespace
  and nodes) and the required label (`Arg`). The table `XARROWS` lists
  the eight commands with their plain arrow glyph.
- **Parse.** After the name, whitespace and a `[` open the optional
  label; it ends at the first `]` outside braces before the end of the
  enclosing group. The required label is read like any command argument
  (a `{group}`, a symbol command or one character, as `\frac`'s). A
  `[` with no `]`, or no required label, makes the command a literal
  drawn in the warning colour; what follows is parsed as usual.
- **Width.** With both labels empty the arrow is the plain arrow glyph:
  same box, same ink as `\rightarrow`, `\mapsto`, …. Otherwise its width
  is the larger of the glyph's advance and the wider label plus
  `XARROW_PAD` (0.5 em of the arrow's size) on each side.
- **Labels** are laid out in script style (`SCRIPT_SCALE`), centred on
  the box, `XARROW_GAP` (0.111 em, TeX's `\bigopspacing1`) above the
  top of the arrow's ink and below its bottom.
- **Drawing a stretched arrow.** The head is the font's own arrow glyph
  at its natural size, at the end it points to (`→` right, `←` left,
  both for `↔`; `⇒`, `⇐` for the double arrows). The shaft is made of
  rules from the other end of the ink to the middle of each head glyph,
  so they overlap the glyph's own shaft:
  - single shafts: one rule of `RULE` thickness on the axis (the bar of
    the font's minus: 0.23 to 0.27 em);
  - double shafts: two rules at the bars of the font's `=` (ink 0.133 to
    0.367 em, so bars centred at 0.153 and 0.347 em), as TeX builds
    `\Rightarrowfill` from `=`;
  - `\xmapsto`: a `→` head, plus a vertical rule as tall as the `↦`
    glyph's ink at its left ink edge;
  - `\xhookrightarrow`: a `→` head, plus a hook of `XARROW_HOOK_SEGMENTS`
    short rules on a half circle above the shaft at the left end, as high
    as the `↪` glyph's ink.
- **Class.** The arrow is a relation (`Rel` spacing); the glyph and the
  shaft sit on the math axis like `\rightarrow`.

## Alternatives considered

- **New extensible glyph pieces in the font subset** — needs a font
  rebuild, and the plan forbids new glyphs outside the subset.
- **Glyph leaders (`−` or `=` tiled, TeX's `\rightarrowfill`)** — many
  glyphs per arrow; rules draw the same bars with fewer quads.
- **Scaling the arrow glyph horizontally** — the renderer has only
  uniform glyph scale, and a stretched head looks wrong.
- **Tail glyphs (`↦`, `↪`) at the left end** — their own head would show
  in the middle of the shaft, since glyphs cannot be clipped.
- **Require braces around the label** — every other command accepts a
  single token; amsmath does too.

## Consequences

- Every exhaustive match on `Node` has one more arm.
- The hook of a stretched `\xhookrightarrow` is a polyline, not the
  font's curve; at text sizes the difference is a pixel or two.
- Nested arrows in labels work through the usual layout recursion,
  bounded by `MAX_DEPTH`.

## Rollback plan

Remove `Node::XArrow`, `Bracket`, `XARROWS` and the `xarrow` layout; the
commands go back to literals. Record it in a new ADR.
