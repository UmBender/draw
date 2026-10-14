# Changelog

All notable changes to draw are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-10-04

First release.

### Added

- Infinite canvas: pan with middle drag or `Space`+drag, zoom at the cursor
  with the wheel (0.05× – 20×), fit everything with `F`, reset with `0`.
- Drawing tools: smoothed pen, line, arrow, rectangle, ellipse; `Shift`
  constrains to squares, circles and 45° angles.
- Grid tool: drag out a table and size it live with the arrow keys (up to
  64 × 64), with optional row and column indices.
- Auto-numbering: new circles, boxes and grid indices are numbered while
  numbering is on; `Shift+N` restarts at 1.
- Bucket fill for shapes and single grid cells; the eraser clears fills or
  whole objects.
- Snapping: alignment guides, outline snapping (arrows meet circles at their
  edge) and an optional snap grid; `Alt` disables snapping while drawing.
- Editing: select, move, `Alt`-drag to duplicate, copy, cut, paste,
  duplicate, select all, delete and clear canvas.
- Unlimited undo and redo of every edit, including clearing the canvas.
- Anti-tremor stroke smoothing with four levels (off, low, medium, high).
- 4× MSAA anti-aliasing (`DRAW_MSAA` to change it).
- Toolbar with clickable tools, colours and helpers, hidden with `Tab`.
- Kanagawa Dragon dark theme with a fixed six-colour palette.
- Keyboard shortcut for every tool and action.
- Idle-friendly event loop (≈ 0 % CPU while idle), layered redraws and
  frame reuse during pan and zoom, smooth with thousands of shapes on an
  8th-gen i5 iGPU (`DRAW_FRAME_TIMES=1` logs frame times).

[Unreleased]: https://github.com/UmBender/draw/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/UmBender/draw/releases/tag/v1.0.0
