---
id: T50
title: Version 2.0.0 refactor
status: todo
wave: 33
branch: task/T50-v2-refactor
depends_on: [T46, T47, T48, T49]
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
with better performance and the bugs found on the way fixed. This is an
epic: the spec step splits it into its own tasks (T51+), each with the
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
  5. **Observability** from [[T47 Always-on diagnostics log]] is in
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

- New user-facing features (separate tasks after or alongside 2.0.0).

## Files owned

- Decided per child task.

## Subtasks (one commit each)

- [ ] spec — `docs(T50): audit the codebase and plan the 2.0.0 tasks`
- [ ] child tasks — listed here once created

## Learning path

Decided per child task.

## Log

- 2026-11-02 — epic written at the owner's request.
