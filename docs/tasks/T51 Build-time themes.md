---
id: T51
title: Build-time themes
status: todo
wave: 32
branch: task/T51-build-time-themes
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task]
---

# T51 Build-time themes

## Goal

Every colour draw shows is set in **one Rust source file**, the theme.
Changing it means editing that file and rebuilding. That is the only
way to change a colour: there is no command-line argument, no
environment variable, no Cargo feature and no configuration file. draw
ships with one theme, the current one (Kanagawa Dragon,
[[ADR-0015 Kanagawa Dragon theme]]). The documentation explains, step
by step, how to change any colour and rebuild. The file must be easy
to read and to change for someone who does not know the codebase,
including a desktop customiser (ricer) who wants draw to match their
other apps.

## Known facts (2026-11-02)

- Most colours are two `const`s in `src/core/palette.rs`: `THEME` (UI
  tokens `bg`, `surface`, `border`, `text`, `accent`, `selection`) and
  `PALETTE` (the six drawing colours: ink, red, green, blue, yellow,
  magenta). `THEME` is read by `shell::{app,render,toolbar}`, `PALETTE`
  by `shell::render` through `palette(ColorId)`.
- Some colours are not in the theme yet. They are derived or tuned
  where they are used, e.g. `shell::render::DOT_GRID_ALPHA`,
  `warn_color`, `label_color`, `shell::app::MARQUEE_FILL_ALPHA`, and
  the colours of the caret, the alignment guides, toolbar icons and
  formula ink. The spec step lists every one (`grep` for colour
  literals, alphas and derived colours in `src/`).
- [[ADR-0012 Fixed palette and theme tokens]] fixes six drawing colours
  chosen by key digit. The theme keeps that count and order, so the
  digit keys keep their meaning.
- Unit tests pin the Kanagawa values (`palette::tests`).
- Owner's decision (2026-11-02): after the 2.0.0 refactor the command
  line takes only basic flags such as `--version` and `--help`
  ([[T50 Version 2.0.0 refactor]]). The theme must not add an option
  of any kind.

## Proposal (settle in the spec step)

1. **One theme file**, e.g. `src/theme.rs` or `src/core/theme.rs`,
   holding nothing but colour constants grouped by where they show:
   canvas, toolbar, selection and marquee, snapping guides and dot grid,
   text caret and edit box, numbering labels, warnings, and the six
   drawing colours. Each constant has a one-line comment saying what it
   colours, written as `0xRRGGBB` hex (plus alpha where it matters),
   so a value can be pasted from any palette.
2. **Every colour configurable**: each colour or alpha found in step 1
   of the spec moves into the theme file. Code elsewhere only reads
   theme constants. A derived colour (e.g. a label that picks dark or
   light text against its fill) either becomes two theme constants or
   keeps its rule but takes its inputs from the theme. The ADR decides
   each case.
3. **Guard test**: no colour literal (`from_hex`, `Color::new`, colour
   alphas) outside the theme file, checked by a test that scans
   `src/`, so new code cannot hard-code a colour again.
4. **Sanity tests on the theme** (they hold for the shipped theme and
   tell a tweaker what broke): the six drawing colours are distinct,
   ink and selection are visible against `bg` (minimum contrast
   ratio). Pinning the exact Kanagawa values stays in a separate test,
   so it is clear which test a tweaker may update.
5. **Documentation**: a README section and a feature note *Themes*
   with: where the file is, what each group colours (ideally with a
   labelled screenshot), how to rebuild (`cargo build --release`,
   `scripts/package.sh`), and a worked example (making the background
   lighter and the ink darker). It says that a light background needs
   a dark ink.

## Spec

To be written in the spec step (ADR required: the theme file's layout,
the list of colours moved into it, the guard test).

## Out of scope

- Shipping more than one theme, or switching themes at run time.
- Choosing a theme through an argument, environment variable, Cargo
  feature or configuration file.
- Changing the number or meaning of the six drawing colours.

## Files owned

- `src/core/palette.rs`, the new theme file
- every file that holds a colour today: `src/shell/{app,render,toolbar,glyphs}.rs`
  and any other the audit finds (listed in the spec)
- `README.md` (theme section)
- new ADR `ADR-T51-*` (amends ADR-0012 and ADR-0015), a feature note
  *Themes*, a tutorial

## Subtasks (one commit each)

- [ ] spec — `docs(T51): …`
- [ ] tests — `test(T51): …`
- [ ] models — `feat(T51): …`
- [ ] behaviour — `feat(T51): …`
- [ ] quality — `chore(T51): …`
- [ ] docs — `docs(T51): …`

## Learning path

Possibly a tutorial on keeping all colours in one place (design tokens,
step 02, revisited) and on colour contrast.

## Log

- 2026-11-02 — note written at the owner's request, one of the last
  features before the 2.0.0 refactor; ships in 1.5.0
  ([[T53 Release 1.5.0]]).
- 2026-11-02 — owner: no build variable, feature or argument. One
  theme ships (the current one). Every colour lives in one source file
  that the docs explain how to edit before compiling.
