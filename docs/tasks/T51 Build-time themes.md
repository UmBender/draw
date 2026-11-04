---
id: T51
title: Build-time themes
status: review
wave: 32
branch: task/T51-build-time-themes
depends_on: []
adrs: ["[[ADR-T51-1 One theme file with every colour]]"]
feature: "[[Themes]]"
tutorial: "[[51 Keeping every colour in one file]]"
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

Decision: [[ADR-T51-1 One theme file with every colour]] (file layout,
constants, derived colours, guard test, contrast threshold).

### Audit (2026-11-04)

`grep` over `src/` for `THEME`, `PALETTE`, `palette(`, `Color::new`,
`Color {`, `Rgba {`, `from_hex`, `from_rgba`, `_ALPHA`, `.a *=`, named
macroquad colours and six-digit hex literals. Every colour draw shows:

| Where | What | Today | Moves to |
|-------|------|-------|----------|
| `core/palette.rs` | six drawing colours | `PALETTE` | `theme::{INK, RED, GREEN, BLUE, YELLOW, MAGENTA}` |
| `core/palette.rs` | six UI tokens | `THEME` | split per place (below); `Theme`/`THEME` removed |
| `shell/app.rs` | canvas background (two `clear_background`, layer clear) | `THEME.bg` | `CANVAS` |
| `shell/app.rs` | caret on a filled shape | `THEME.bg` | `TEXT_ON_FILL` |
| `shell/app.rs` | marquee fill | `THEME.selection` × `MARQUEE_FILL_ALPHA = 0.25` | `MARQUEE_FILL`, `MARQUEE_FILL_ALPHA = 63` |
| `shell/app.rs` | marquee outline | `THEME.accent` | `MARQUEE_OUTLINE` |
| `shell/app.rs` | layer blit tint | macroquad `WHITE` | `Rgba::NO_TINT` (not a colour) |
| `shell/render.rs` | dot grid | `THEME.border` × `DOT_GRID_ALPHA = 0.35` | `DOT_GRID`, `DOT_GRID_ALPHA = 89` |
| `shell/render.rs` | alignment guides | `THEME.accent` | `GUIDE` |
| `shell/render.rs` | selection / edit box outline | `THEME.accent` | `SELECTION_OUTLINE` |
| `shell/render.rs` | labels and text on a fill (`label_color`) | `THEME.bg` | `TEXT_ON_FILL` (rule kept) |
| `shell/render.rs` | literal formula source (`warn_color`) | `PALETTE[1]`, `PALETTE[4]` | `FORMULA_SOURCE`, `FORMULA_SOURCE_ALT` (rule kept) |
| `shell/toolbar.rs` | strip and flyout background | `THEME.surface` | `TOOLBAR_BACKGROUND` |
| `shell/toolbar.rs` | strip edge, flyout buttons | `THEME.border` | `TOOLBAR_BORDER` |
| `shell/toolbar.rs` | letters and labels | `THEME.text` | `TOOLBAR_TEXT` |
| `shell/toolbar.rs` | active tool, current colour, pen badge | `THEME.accent` | `TOOLBAR_ACTIVE` |
| `shell/toolbar.rs` | fill behind an active toggle | `THEME.selection` | `TOOLBAR_ACTIVE_FILL` |
| `shell/toolbar.rs` | disabled undo/redo | `DISABLED_ALPHA = 0.3` | `TOOLBAR_DISABLED_ALPHA = 76` (rule kept) |
| `shell/glyphs.rs` | atlas texels `[255, 255, 255, coverage]` | mask | stays: not a colour |

No other file in `src/` holds a colour. Files touched beyond the list in
*Files owned*: `src/lib.rs` (one `pub mod theme;` line, needed by the
owner's choice of `src/theme.rs`) and `docs/architecture/Theme.md`
(holds the colours and names the file they live in).

### Acceptance criteria

- **AC-1 One file.** Every colour, alpha and derived colour's input is a
  `pub const` in `src/theme.rs`, grouped by where it shows, each with a
  one-line comment, as `0xRRGGBB` plus a separate `u8` alpha. —
  `theme::tests::guard_finds_no_colour_literal_outside_theme`, and the
  scanner's own tests `scanner_flags_hex_colour_literal`,
  `scanner_ignores_hashes_and_code_points`,
  `scanner_flags_colour_constructor_with_number`,
  `scanner_allows_constructor_with_variable`,
  `scanner_flags_struct_literal_and_alpha_edit`,
  `scanner_flags_macroquad_named_colour`,
  `scanner_skips_comments_and_test_modules`.
- **AC-2 Kanagawa unchanged.** The shipped values are those of ADR-0015,
  and `PALETTE` reads the six drawing constants in key order. —
  `theme::tests::theme_matches_kanagawa_dragon`,
  `palette::tests::palette_reads_drawing_colours_from_theme`,
  `palette::tests::palette_matches_documented_drawing_colours` (kept).
- **AC-3 Sanity.** The six drawing colours are distinct; `INK` and
  `SELECTION_OUTLINE` contrast at least 3:1 with `CANVAS`. —
  `theme::tests::drawing_colours_are_distinct`,
  `theme::tests::ink_contrasts_with_canvas`,
  `theme::tests::selection_outline_contrasts_with_canvas`.
- **AC-4 Colour helpers.** `Rgba::from_hex_alpha` sets the alpha byte;
  `Rgba::faded` scales alpha by `alpha / 255` (opaque → exactly `alpha`,
  `255` → unchanged); `relative_luminance` is 0 for black and 1 for
  white; `contrast_ratio` is 21 for black on white, 1 for a colour on
  itself, symmetric and in `1..=21`. —
  `palette::tests::from_hex_alpha_keeps_channels_and_sets_alpha`,
  `faded_opaque_gives_alpha`, `faded_by_opaque_is_identity`,
  `no_tint_is_opaque_white`, `relative_luminance_of_black_and_white`,
  `contrast_ratio_black_on_white_is_21`,
  `contrast_ratio_symmetric_and_bounded` (proptest).
- **AC-5 Same pixels.** The new alphas give the same vertex bytes as the
  old `f32` factors, and the shell's colours equal the old tokens. —
  `render::tests::theme_alphas_give_previous_vertex_bytes`,
  `render::tests::faded_text_gives_previous_vertex_bytes`.
- **AC-6 Derived rules kept.** Text on a filled shape is `TEXT_ON_FILL`,
  on an empty one its outline colour; literal source is `FORMULA_SOURCE`,
  or `FORMULA_SOURCE_ALT` when the text is drawn in `FORMULA_SOURCE`. —
  `render::tests::label_color_on_fill_is_text_on_fill`,
  `render::tests::label_color_without_fill_is_outline`,
  `render::tests::warn_color_uses_theme_formula_source` (plus the
  existing `warn_color` tests).
- **AC-7 No option.** Nothing reads an argument, environment variable,
  feature or file to pick colours: `src/theme.rs` has no functions and
  no `use` outside tests (reviewed; the guard keeps colours there).

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

- [x] spec — `docs(T51): …`
- [x] tests — `test(T51): …`
- [x] models — `feat(T51): …`
- [x] behaviour — `feat(T51): …`
- [x] quality — `chore(T51): …`
- [x] docs — `docs(T51): …`

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
- 2026-11-04 — batch run: the task branch already existed with no task
  commits (it pointed at an older `main`); fast-forwarded it to `main`
  (`d53413a`) instead of creating it.
- 2026-11-04 — a throwaway probe test (`tests/probe_tmp.rs`, measuring
  how macroquad turns `Color` into vertex bytes) could not be deleted with
  the allowed commands; it is kept in the stash
  `batch-T51-2026-11-04-probe` and can be dropped.
- 2026-11-04 — UNATTENDED CHOICE: `src/lib.rs` gets `pub mod theme;`
  (the owner put the file at the crate root) and
  `docs/architecture/Theme.md` is updated to name `src/theme.rs`; both
  are outside the original *Files owned*.
- 2026-11-04 — UNATTENDED CHOICE: the "selection" contrast check is on
  `SELECTION_OUTLINE` (`#7fa8bc`, the outline that shows a selection).
  The marquee fill `#2d4f67` is a 25 % tint and would fail 3:1 against
  `#181616` (about 2:1) by design.
- 2026-11-04 — UNATTENDED CHOICE: shared tokens are split per place
  (e.g. `accent` → `SELECTION_OUTLINE`, `MARQUEE_OUTLINE`, `GUIDE`,
  `TOOLBAR_ACTIVE`), see ADR-T51-1.
- 2026-11-04 — behaviour: helpers implemented, every shell colour reads
  `crate::theme`; `cargo test` green.
- 2026-11-04 — quality: `scripts/check.sh` green after rustfmt and clippy
  fixes in the new tests (`unreadable_literal`, `expect_used`,
  `assert_is_empty`, `type_complexity`).
- 2026-11-04 — docs: feature note [[Themes]], tutorial
  [[51 Keeping every colour in one file]], README *Theme* section,
  [[Theme]] architecture note rewritten to the new constants.
- 2026-11-04 — NOT DONE (outside *Files owned*): the T02 feature note
  `docs/features/Theme and palette.md` still describes `THEME` and
  `core::palette` as the place colours live; the integrator should point
  it at [[Themes]]. The labelled screenshot was not made.
