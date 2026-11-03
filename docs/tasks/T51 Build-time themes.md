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

Whoever builds draw picks its colours when compiling: one of several
built-in themes, or their own, without editing code spread across the
crate. A user who customises their desktop (ricing) can build draw to
match it. There is still no configuration file and nothing is read at
run time. The default build looks exactly like today (Kanagawa Dragon,
[[ADR-0015 Kanagawa Dragon theme]]).

## Known facts (2026-11-02)

- The colours are two `const`s in `src/core/palette.rs`: `THEME` (UI
  tokens `bg`, `surface`, `border`, `text`, `accent`, `selection`) and
  `PALETTE` (the six drawing colours: ink, red, green, blue, yellow,
  magenta). `THEME` is read by `shell::{app,render,toolbar}`, `PALETTE`
  by `shell::render` through `palette(ColorId)`.
- [[ADR-0012 Fixed palette and theme tokens]] fixes six drawing colours
  chosen by key digit. A theme must keep that count and order, so the
  digit keys keep their meaning across themes.
- The ink colour has to contrast with `bg`. A light theme therefore
  needs its own ink (and maybe darker accents), so a theme is `THEME`
  **and** `PALETTE` together.
- Unit tests pin the Kanagawa values (`palette::tests`). They have to
  hold for the default build.
- [[ADR-0014 Minimal dependencies and no unsafe]]: no new dependency.

## Proposal (settle in the spec step)

1. **How the theme is chosen** (the ADR decides):
   - `DRAW_THEME=<name> cargo build --release`, read with
     `option_env!` in a `const` match. Cargo rebuilds when the variable
     changes. No `build.rs`, and an unknown name is a compile error
     (`const` panic) that lists the known ones. Or:
   - Cargo features `theme-<name>`. These are idiomatic, but features
     are additive, so two themes at once must be a compile error.
   - Either way the default (nothing set) is Kanagawa Dragon.
2. **Custom themes**: `DRAW_THEME_FILE=/path/theme.rs`, or a documented
   recipe (copy one theme block in `palette.rs`, give it a name,
   rebuild). Decide whether this is worth more than the recipe. Parsing
   a TOML/Alacritty file at build time would need a `build.rs`, and
   that is out of scope.
3. **Built-in set**: Kanagawa Dragon (default), plus a few well-known
   dark and light themes with permissive licences (e.g. Gruvbox dark,
   Catppuccin Mocha/Latte, Solarized light, Nord). Check each palette's
   licence before copying values.
4. **Checks for every theme** (a test over all built-in themes): ink
   and every drawing colour reach a minimum contrast with `bg`
   (WCAG-style ratio), `selection` is visible on `bg`, and the six
   colours are distinct.
5. `draw --version`, or the T47 start-up line, names the theme the
   binary was built with, so bug reports say which colours they show.
6. `scripts/package.sh`, the Flatpak and the release keep the default.
   The README gets a "Build with another theme" section.

## Spec

To be written in the spec step (ADR required: selection mechanism,
theme set, contrast rule).

## Out of scope

- Choosing or switching the theme at run time, or a config file.
- Changing the number or meaning of the six drawing colours.
- Reading Alacritty/Kitty theme files.

## Files owned

- `src/core/palette.rs` (and a new `src/core/themes.rs` if the themes
  get their own file)
- `src/shell/diag/mod.rs` (theme in the start-up line)
- `README.md` (build section)
- new ADR `ADR-T51-*`, a feature note *Themes*, a tutorial

## Subtasks (one commit each)

- [ ] spec — `docs(T51): …`
- [ ] tests — `test(T51): …`
- [ ] models — `feat(T51): …`
- [ ] behaviour — `feat(T51): …`
- [ ] quality — `chore(T51): …`
- [ ] docs — `docs(T51): …`

## Learning path

Possibly a tutorial on compile-time configuration in Rust (`option_env!`,
`const` evaluation, Cargo features) and on colour contrast.

## Log

- 2026-11-02 — note written at the owner's request, one of the last
  features before the 2.0.0 refactor; ships in 1.5.0
  ([[T53 Release 1.5.0]]).
