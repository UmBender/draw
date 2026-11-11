---
id: ADR-T110-1
title: Minimap geometry, routing and overview cache
status: accepted
kind: decision
date: 2026-11-10
task: T110
builds_on: ["[[ADR-0003 Headless core and thin shell]]", "[[ADR-0006 Redraw on demand]]", "[[ADR-T08-1 Tool context and gesture overlay]]", "[[ADR-T24-1 Document layer keyed by a document revision]]", "[[ADR-T24-2 Overlay drawn straight to the window]]", "[[ADR-T74-1 Line-based editor snapshot format]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T110-1 Minimap geometry, routing and overview cache

## Context

[[T110 Minimap]] shows an overview of the drawing while panning. The
owner fixed when it shows (`Space` held, or a middle-button pan), where
(bottom-right) and that a click or drag in it moves the view. Open were
its size, what world area it shows, how a drag in it stays stable, where
the input routing lives, how the overview is cached, and how it looks at
scales far below the camera's smallest zoom.

## Decision

**Panel.** 200 × 150 px (`PANEL_SIZE`), 12 px (`MARGIN_PX`) from the
right and bottom edges, with 6 px (`PADDING_PX`) of padding around the
drawing area. A viewport that cannot hold the panel and its margins has
no minimap. The panel uses the toolbar's background (85 % opaque) and
border colours, and the selection colour for the view frame; no new
theme token.

**World box.** The minimap shows the union of every shape's bounds and
the visible world rectangle, scaled uniformly to fit the drawing area
and centred in it (`core::minimap::Fit`). The mapping is computed in
`f64` and every result is clamped to its target rectangle, so content at
`±1e38` or a box with no extent still gives finite coordinates. The view
frame is at least 4 px on each side so it stays visible on a large
drawing.

**When it shows.** `Editor::minimap` returns the geometry while `Space`
is held with no text block open, while a pan started with the middle
button runs, and while a drag in the minimap runs. A left drag with the
hand tool pans without it: the owner listed `Space` and the middle
button only. `Editor::handle` returns `true` when the minimap appears or
disappears, so the shell redraws the overlay for a `Space` press that
changes nothing else. A repeated `Space` key changes nothing and asks
for no frame.

**Routing in the editor.** A left press inside the panel while `Space`
is held (and the minimap shows) starts a new gesture, `Gesture::Minimap`,
instead of a pan: the view is centred on the world point under the
pointer, on the press, on every move and on the release. The press never
reaches a tool. The gesture freezes the world box it started with:
centring the view near an edge would otherwise grow the box, rescale the
minimap under the pointer and make the view run away. A pointer outside
the drawing area is clamped to it. From outside, the gesture reports as
`ActiveGesture::Pan`. The routing is in `core::editor`, not the shell,
because it depends on `Space`, the text edit mode and the one-gesture
rule, which the editor owns and the fuzzer drives.

**Snapshot.** The `gesture` record gains the form
`minimap <button> <min> <max>`. Like the `select resize` record of
[[ADR-T91-1 Resize handles and scale rules]], it is additive: every
existing snapshot reads as before and no fixture changes, so
`snapshot::VERSION` stays (UNATTENDED CHOICE).

**Content bounds.** `Editor` keeps the content bounds with the document
revision they were computed for. They are refreshed at the end of
`Editor::handle` only while the minimap shows and only when the revision
differs. While hidden nothing is computed. The cache is not saved state.

**Overview cache in the shell.** `shell::minimap` draws the shapes into
a render target of the drawing area's size, through a `Fit` of the
content bounds alone. Its key is the document revision, shape count,
content bounds and target size; the camera is not part of it. The
overlay then draws the panel, the cached texture at the place the
content has in the current world box (`Minimap::place`), and the frame.
Panning changes only that placement and the frame.

**Schematic drawing.** The overview is not the document renderer: the
camera cannot go below 5 % zoom, and a full render would drop every
line thinner than a pixel. Strokes, lines and outlines are drawn 1 px
wide in their colour, fills as filled areas, text blocks as bars of
their box, grids as their outline and fills, and a shape smaller than
2 px as a dot. When the content takes less than 12 px of the panel (the
view is far away or far larger), a small filled marker is drawn over it,
since a texture scaled down that far can lose its lines.

## Alternatives considered

- **Re-render the overview whenever the world box changes** — sharp at
  every scale, but it would re-render on every pan step outside the
  content, against AC-3.
- **Render with the document renderer through a scaled projection** —
  shows text and exact widths, but lines vanish below 1 px and the
  camera's zoom clamp gets in the way.
- **Routing in the shell, in front of the editor** (as the toolbar) —
  the shell would need `Space`, text edit and gesture state, and the
  fuzzer could not reach it.
- **A live world box during a minimap drag** — simpler, but the view
  runs away at the edges.
- **Showing it for the hand tool too** — consistent with "while
  panning", but not what the owner chose.
- **Bumping `snapshot::VERSION`** — not needed for an additive record,
  and it would collide with the bumps of T107 and T116 in this batch.

## Consequences

- The minimap costs nothing while hidden and one textured quad plus a
  frame per presented frame while shown.
- The overview is schematic: no glyphs, no labels, uniform 1 px lines.
- A snapshot taken during a minimap drag is not readable by older
  builds (they report an unknown gesture).
- `ActiveGesture::Pan` now also covers a drag in the minimap.

## Rollback plan

Remove `core::minimap`, `shell::minimap`, `Gesture::Minimap` and the
content cache from `core::editor`, and the calls in `shell::app`; add a
rollback ADR. Snapshots taken during a minimap drag stop loading.
