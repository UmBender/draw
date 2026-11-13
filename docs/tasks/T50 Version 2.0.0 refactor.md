---
id: T50
title: Version 2.0.0 refactor
status: in-progress
wave: 63
branch: task/T50-v2-refactor
depends_on: [T46, T47, T48, T49, T51, T52, T53, T118, T127, T128, T129, T130, T131, T132, T133, T134, T135, T136, T137, T138, T139, T140, T141, T142, T143, T144, T145, T146, T147, T148, T149, T151, T152, T153, T154, T155, T156, T157, T158, T159, T160, T161]
adrs: []
feature: ""
tutorial: ""
tags: [task, epic]
---

# T50 Version 2.0.0 refactor

## Goal

draw 2.0.0: the X11/Xwayland backend deprecated in
[[T46 Native Wayland by default and Xwayland deprecated]] is removed,
and the codebase is reshaped around what the app has become since 1.0
(text and formulas, pen tablets, native Wayland, layered rendering),
with better performance and the bugs found on the way fixed. 2.0.0 is
a *finished* version: it adds no features. Every feature it has ships
first in 1.5.0 ([[T53 Release 1.5.0]]), and after 2.0.0 only bug fixes
follow. This is an epic: the spec step splits it into its own tasks (T51+), each with the
usual SDD/TDD workflow.

## Known facts (2026-11-02)

- Owner's goals for 2.0.0:
  1. **Remove X11/Xwayland**: `LinuxBackend::X11Only` and the fallback,
     the X11 paths in preflight and packaging, the X11 pen pressure
     code, the `x11` Flatpak socket. Remodel the shell for one backend.
  2. **Align the codebase with everything since the first version**:
     the architecture of [[Architecture]] was set in T00–T14; text
     (T29–T33, T36–T39, T41), shape text (T31), tablets (T34, T35, T40,
     T42), Wayland (T28, T44) and redraw layers (T24–T26) were added on
     top of it.
  3. **Performance**: open items include the X11 moved-frame overrun of
     ADR-T26-2 found in T28 AC-6 (gone with X11) and the budgets of
     ADR-T24-5 / ADR-T26-2 on the target.
  4. **Bugs**: findings of [[T45 Doubled keys after a long idle on
     Wayland]], [[T48 Clean exit on Ctrl-C]], [[T49 Long-running
     stability]].
  5. **Command line**: only basic flags such as `--version` and
     `--help` (owner, 2026-11-02). No option changes the app's
     behaviour or looks; the colours are set before compiling
     ([[T51 Build-time themes]]). The `DRAW_*` environment variables
     stay, but **only for debugging** (owner, 2026-11-02): normal use
     never needs one, and none is part of the user-facing docs except
     a "Debugging" section. The audit sorts today's variables:
     - debugging, kept: `DRAW_LOG` (T47), `DRAW_FRAME_TIMES`,
       `DRAW_SKIP_PREFLIGHT`;
     - removed with what they switch: `DRAW_BACKEND` (X11 goes away),
       `DRAW_EXPERIMENTAL` (emptied by [[T52 Text in shapes by default]]);
     - to decide: `DRAW_MSAA` (a weak-GPU workaround) and `DRAW_TABLET`
       (pen mode override when auto-detection is wrong). Either the
       default becomes right without them and they stay as debugging
       overrides, or the audit finds another way.
  6. **Observability** from [[T47 Always-on diagnostics log]] is in
     place before the refactor, so regressions are visible.
- About 28.7k lines in `src/core` and `src/shell`, plus the vendored
  miniquad (`vendor/miniquad`, patched per ADR-T35-1, ADR-T40-1,
  ADR-T44-1). Dropping X11 may make a slimmer Wayland-only backend
  possible (decide in an ADR: keep patching miniquad vs own Wayland
  layer).
- Rollbacks stay new ADRs plus `git revert`; no history rewrite.

## Proposal (settle in the spec step)

1. **Audit** (spec step): module map, size and coupling of `editor.rs`,
   `app.rs`, `render.rs`, the tool modules; duplicated paths; dead code
   and switches left from experiments (`core::experimental`, shape-text
   switch of ADR-T31-1); test and fuzz coverage gaps; perf profile on
   the target. Output: an architecture ADR for 2.0 and a list of tasks.
2. **Order**: X11 removal first (smallest risk, shrinks the surface),
   then the shell remodel, then core refactors, then perf work measured
   against the budgets, each behind green `scripts/check.sh` and the
   proptest fuzz suite.
3. **Release**: `CHANGELOG.md` `### Removed` for X11/Xwayland, version
   2.0.0, packages and Flatpak without `x11`.

## Spec

The spec step of this epic produces the audit, the 2.0 ADR and the
child tasks; it does not implement code.

## Out of scope

- New user-facing features. The last ones (build-time themes,
  [[T51 Build-time themes]]; text in shapes by default,
  [[T52 Text in shapes by default]]) ship in 1.5.0. After 2.0.0 only
  bug fixes follow (owner's decision, 2026-11-02).

## Files owned

- Decided per child task.

## Subtasks (one commit each)

- [x] spec — `docs(T50): propose the 2.0 architecture` ([[2.0 Architecture proposal]])
- [x] child tasks — created 2026-11-12, in the order below

**Audit and rails**

- [ ] [[T127 2.0 audit and baseline]]
- [ ] [[T128 Keep X11 and revert the deprecation]]
- [ ] [[T129 Release gate with end-to-end on X11 and Wayland]]
- [ ] [[T130 Coverage in CI with a ratchet]]
- [ ] [[T131 No dynamic dispatch and stricter lints]]
- [ ] [[T132 Workspace split with unsafe levels]]

**Types and the automaton**

- [ ] [[T133 Typed units and ids]]
- [ ] [[T134 The control automaton states, inputs and effects]]
- [ ] [[T135 Automaton modal states, pages, camera and selection]]
- [ ] [[T136 Automaton gestures and text editing per tool]]
- [ ] [[T137 Automaton effects queue and time as an input]]
- [ ] [[T138 Model-based tests of the automaton]]
- [ ] [[T139 Split tex.rs into modules]]

**Platform traits and shell tests**

- [ ] [[T140 Platform and Gpu traits with recording interpreters]]
- [ ] [[T141 Fuzz of the shell through the recording platform]]

**Own backend**

- [ ] [[T142 draw-sys foundation with dlopen, safe wrappers and the unsafe test set]]
- [ ] [[T145 OpenGL ES 3.0 context and loader (EGL and GLX)]]
- [ ] [[T143 Wayland backend with window, keyboard, pointer, tablet and clipboard]]
- [ ] [[T144 X11 backend with window, keyboard, pointer, XInput2 pressure and selections]]

**Renderer, pacing, threads**

- [ ] [[T146 Retained renderer in draw-gfx]]
- [ ] [[T147 Frame pacing by the compositor and idle at zero]]
- [ ] [[T148 Multithreaded tessellation, only with a measured gain]]
- [ ] [[T152 Command line and DRAW_* variables audit]]

**Cutover and release**

- [ ] [[T149 Cutover removing miniquad and macroquad]]
- [ ] [[T151 Release 2.0.0]]

**Optional**

- [ ] [[T150 Optional Vulkan backend for the Gpu trait]]

## Learning path

Decided per child task.

## Log

- 2026-11-02 — epic written at the owner's request.
- 2026-11-02 — owner: 2.0.0 adds no features and is followed only by bug
  fixes. Starts after the 1.5.0 release (T53), which carries T46–T49,
  T51 and T52.
- 2026-11-02 — owner: the 2.0.0 command line takes only basic flags
  (`--version`, `--help`). The `DRAW_*` variables stay, for debugging
  only.
- 2026-11-03 — 1.5.0 (T53) now also carries T54–T72; T50 moves to
  wave 52.
- 2026-11-12 — owner's decisions after the 1.6.0 release (see [[2.0 Architecture proposal]]): **X11 stays** (goal 1 above is reversed and the deprecation of T46 is reverted by a new ADR); the patched miniquad, and so macroquad, are replaced by a backend of our own; the release depends on the e2e suite; `unsafe` is allowed in confined modules with full unit, integration and regression tests per primitive; threads only with a measured gain; all the quality proposals are accepted; no dynamic dispatch where avoidable, with tagless final, GADT-like encodings and type states; the control engine is an automaton. Still no new features.
- 2026-11-12 — owner: OpenGL ES 3.0 for 2.0.0 and a Vulkan backend as an **optional** child task, conditioned on a measurement (see *OpenGL or Vulkan* in [[2.0 Architecture proposal]]); the `Gpu` trait is designed for both.
- 2026-11-12 — child tasks T127–T152 created after the owner approved the removal of macroquad with the miniquad and the `dlopen` of the C libraries. T150 (Vulkan) is optional and does not block T151.
