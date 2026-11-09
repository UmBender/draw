---
tags: [moc, task]
---

# Task Board

Status lives in each task note's frontmatter; this board is refreshed by the
integrator on `main` (see [[Parallel Execution]]).

## Dependency graph

```mermaid
flowchart TD
  T00[T00 Bootstrap] --> T01[T01 Geometry]
  T00 --> T02[T02 Palette]
  T01 --> T03[T03 Camera]
  T01 --> T04[T04 Smoothing]
  T01 --> T05[T05 Shape model]
  T02 --> T05
  T05 --> T06[T06 Document + history]
  T03 --> T07[T07 Renderer]
  T05 --> T07
  T03 --> T08[T08 Editor core]
  T06 --> T08
  T08 --> T09[T09 Creation tools]
  T04 --> T09
  T08 --> T10[T10 Editing tools]
  T08 --> T11[T11 Keymap + macros]
  T07 --> T12[T12 App shell + toolbar]
  T09 --> T12
  T10 --> T12
  T11 --> T12
  T09 --> T13[T13 Fuzz harness]
  T10 --> T13
  T11 --> T13
  T12 --> T14[T14 Perf + release validation]
  T13 --> T14
  T12 --> T15[T15 Anti-aliasing]
  T12 --> T16[T16 Shape model v2 + helper skeleton]
  T13 --> T16
  T16 --> T17[T17 Snapping]
  T16 --> T18[T18 Grid tool]
  T17 --> T19[T19 Auto-numbering]
  T17 --> T20[T20 Outline snapping]
  T15 --> T14
  T17 --> T14
  T18 --> T14
  T19 --> T14
  T20 --> T14
  T18 --> T21[T21 Cell fill + fill eraser]
  T21 --> T14
  T19 --> T22[T22 Numbering drives grid indices]
  T21 --> T22
  T22 --> T14
  T05 --> T23[T23 Arrow head proportional to width]
  T23 --> T14
  T14 --> T24[T24 Smooth redraws with large documents]
  T14 --> T25[T25 Linear-time selection with large documents]
  T24 --> T26[T26 Pan and zoom without re-rendering]
  T14 --> T27[T27 Linux packaging]
  T27 --> T28[T28 Experimental native Wayland backend]
  T14 --> T29[T29 Text blocks]
  T29 --> T32[T32 Fast text rendering]
  T32 --> T30[T30 Basic TeX formulas]
  T29 --> T30
  T29 --> T31[T31 Text in rectangles and grid cells]
  T30 --> T33[T33 Math font for formulas]
  T28 --> T34[T34 Pen tablet input]
  T34 --> T35[T35 Pen pressure and native Wayland tablets]
  T33 --> T36[T36 Integrals in formulas]
  T31 --> T37[T37 Remember the text size]
  T12 --> T38[T38 Left-hand tool keys]
  T36 --> T39[T39 Accents in formulas]
  T35 --> T40[T40 Pen pressure on native Wayland]
  T36 --> T41[T41 Integral superscript at the advance]
  T35 --> T42[T42 Join pen tip bounces]
  T43[T43 Adopt clippy 1.99 lints]
  T28 --> T44[T44 One keyboard per Wayland seat]
  T44 --> T45[T45 Doubled keys after a long idle on Wayland]
  T45 --> T46[T46 Native Wayland by default and Xwayland deprecated]
  T47[T47 Always-on diagnostics log] --> T48[T48 Clean exit on Ctrl-C]
  T47 --> T49[T49 Long-running stability]
  T51[T51 Build-time themes]
  T52[T52 Text in shapes by default]
  T46 --> T53[T53 Release 1.5.0]
  T47 --> T53
  T48 --> T53
  T49 --> T53
  T51 --> T53
  T52 --> T53
  T54[T54 Math alphabet styles]
  T54 --> T53
  T54 --> T55[T55 Math font subset for alphabets and symbols]
  T55 --> T53
  T54 --> T56[T56 Bold math]
  T55 --> T56[T56 Bold math]
  T56 --> T53
  T54 --> T57[T57 Blackboard bold]
  T55 --> T57[T57 Blackboard bold]
  T57 --> T53
  T54 --> T58[T58 Fraktur letters]
  T55 --> T58[T58 Fraktur letters]
  T58 --> T53
  T54 --> T59[T59 Script letters]
  T55 --> T59[T59 Script letters]
  T59 --> T53
  T54 --> T60[T60 Sans-serif and typewriter letters]
  T55 --> T60[T60 Sans-serif and typewriter letters]
  T60 --> T53
  T61[T61 Hebrew letters]
  T61 --> T53
  T54 --> T62[T62 Complete Greek letters]
  T62 --> T53
  T63[T63 Trigonometric and hyperbolic functions]
  T63 --> T53
  T63 --> T64[T64 Named operators with limits]
  T64 --> T53
  T55 --> T65[T65 Set and logic symbols]
  T65 --> T53
  T55 --> T66[T66 Big set and logic operators]
  T65 --> T66[T66 Big set and logic operators]
  T66 --> T53
  T55 --> T67[T67 Algebra and group theory symbols]
  T64 --> T67[T67 Algebra and group theory symbols]
  T67 --> T53
  T55 --> T68[T68 Category theory symbols]
  T67 --> T68[T68 Category theory symbols]
  T68 --> T53
  T55 --> T69[T69 Classical physics notation]
  T56 --> T69[T69 Classical physics notation]
  T69 --> T53
  T69 --> T70[T70 Quantum notation]
  T70 --> T53
  T55 --> T71[T71 Combinatorics notation]
  T71 --> T53
  T72[T72 Feature proposal]
  T72 --> T53
  T73[T73 Continuous integration on GitHub]
  T73 --> T53
  T74[T74 Editor snapshots for tests]
  T74 --> T53
  T73 --> T75[T75 End-to-end test suite]
  T74 --> T75
  T75 --> T53
  T74 --> T76[T76 Remembered text size past the ladder]
  T76 --> T53
  T77[T77 Test sessions that leave the machine usable] --> T53
  T74 --> T78[T78 Formula ink outside its text box]
  T78 --> T53
  T75 --> T79[T79 Wayland keyboard without a keymap]
  T79 --> T53
  T75 --> T80[T80 Mouse cursor in end-to-end screenshots]
  T80 --> T53
  T53 --> T50[T50 Version 2.0.0 refactor]
```

## Waves (tasks in one wave run in parallel)

| Wave | Tasks |
|------|-------|
| 0 | [[T00 Bootstrap]] |
| 1 | [[T01 Geometry primitives]] · [[T02 Palette and theme tokens]] |
| 2 | [[T03 Camera]] · [[T04 Stroke smoothing]] · [[T05 Shape model]] |
| 3 | [[T06 Document and history]] · [[T07 Renderer]] |
| 4 | [[T08 Editor core and input model]] |
| 5 | [[T09 Creation tools]] · [[T10 Editing tools]] · [[T11 Keymap and macros]] |
| 6 | [[T12 App shell and toolbar]] · [[T13 Fuzz harness]] |
| 7 | [[T15 Anti-aliasing]] · [[T16 Shape model v2 and helper skeleton]] |
| 8 | [[T17 Snapping]] · [[T18 Grid tool]] |
| 9 | [[T19 Auto-numbering]] · [[T20 Outline snapping]] |
| 10 | [[T21 Cell fill and fill eraser]] |
| 11 | [[T22 Numbering drives grid indices]] · [[T23 Arrow head proportional to width]] |
| 12 | [[T14 Performance and release validation]] |
| 13 | [[T24 Smooth redraws with large documents]] · [[T25 Linear-time selection with large documents]] |
| 14 | [[T26 Pan and zoom without re-rendering]] |
| 15 | [[T29 Text blocks]] |
| 16 | [[T32 Fast text rendering]] |
| 17 | [[T30 Basic TeX formulas]] |
| 18 | [[T33 Math font for formulas]] |
| 19 | [[T31 Text in rectangles and grid cells]] |
| 20 | [[T27 Linux packaging]] |
| 21 | [[T28 Experimental native Wayland backend]] |
| 22 | [[T34 Pen tablet input]] |
| 23 | [[T35 Pen pressure and native Wayland tablets]] |
| 24 | [[T36 Integrals in formulas]] · [[T37 Remember the text size]] · [[T38 Left-hand tool keys]] |
| 25 | [[T39 Accents in formulas]] |
| 26 | [[T40 Pen pressure on native Wayland]] |
| 27 | [[T41 Integral superscript at the advance]] |
| 28 | [[T42 Join pen tip bounces]] · [[T43 Adopt clippy 1.99 lints]] |
| 29 | [[T44 One keyboard per Wayland seat]] |
| 30 | [[T45 Doubled keys after a long idle on Wayland]] |
| 31 | [[T46 Native Wayland by default and Xwayland deprecated]] · [[T47 Always-on diagnostics log]] |
| 32 | [[T48 Clean exit on Ctrl-C]] · [[T49 Long-running stability]] · [[T51 Build-time themes]] · [[T52 Text in shapes by default]] |
| 33 | [[T54 Math alphabet styles]] · [[T72 Feature proposal]] · [[T73 Continuous integration on GitHub]] · [[T74 Editor snapshots for tests]] · [[T77 Test sessions that leave the machine usable]] |
| 34 | [[T55 Math font subset for alphabets and symbols]] · [[T75 End-to-end test suite]] · [[T76 Remembered text size past the ladder]] · [[T78 Formula ink outside its text box]] |
| 35 | [[T56 Bold math]] · [[T79 Wayland keyboard without a keymap]] · [[T80 Mouse cursor in end-to-end screenshots]] |
| 36 | [[T57 Blackboard bold]] |
| 37 | [[T58 Fraktur letters]] |
| 38 | [[T59 Script letters]] |
| 39 | [[T60 Sans-serif and typewriter letters]] |
| 40 | [[T61 Hebrew letters]] |
| 41 | [[T62 Complete Greek letters]] |
| 42 | [[T63 Trigonometric and hyperbolic functions]] |
| 43 | [[T64 Named operators with limits]] |
| 44 | [[T65 Set and logic symbols]] |
| 45 | [[T66 Big set and logic operators]] |
| 46 | [[T67 Algebra and group theory symbols]] |
| 47 | [[T68 Category theory symbols]] |
| 48 | [[T69 Classical physics notation]] |
| 49 | [[T70 Quantum notation]] |
| 50 | [[T71 Combinatorics notation]] |
| 51 | [[T53 Release 1.5.0]] |
| 52 | [[T50 Version 2.0.0 refactor]] |

## Status

| Task | Status | Depends on | Branch |
|------|--------|------------|--------|
| [[T00 Bootstrap]] | done | — | `task/T00-bootstrap` |
| [[T01 Geometry primitives]] | done | T00 | `task/T01-geometry` |
| [[T02 Palette and theme tokens]] | done | T00 | `task/T02-palette` |
| [[T03 Camera]] | done | T01 | `task/T03-camera` |
| [[T04 Stroke smoothing]] | done | T01 | `task/T04-smoothing` |
| [[T05 Shape model]] | done | T01, T02 | `task/T05-shape` |
| [[T06 Document and history]] | done | T05 | `task/T06-document` |
| [[T07 Renderer]] | done | T03, T05 | `task/T07-renderer` |
| [[T08 Editor core and input model]] | done | T03, T06 | `task/T08-editor` |
| [[T09 Creation tools]] | done | T04, T08 | `task/T09-creation-tools` |
| [[T10 Editing tools]] | done | T08 | `task/T10-editing-tools` |
| [[T11 Keymap and macros]] | done | T08 | `task/T11-keymap` |
| [[T12 App shell and toolbar]] | done | T07, T09, T10, T11 | `task/T12-app-shell` |
| [[T13 Fuzz harness]] | done | T09, T10, T11 | `task/T13-fuzz` |
| [[T15 Anti-aliasing]] | done | T12 | `task/T15-antialiasing` |
| [[T16 Shape model v2 and helper skeleton]] | done | T12, T13 | `task/T16-helper-skeleton` |
| [[T17 Snapping]] | done | T16 | `task/T17-snapping` |
| [[T18 Grid tool]] | done | T16 | `task/T18-grid-tool` |
| [[T19 Auto-numbering]] | done | T16, T17 | `task/T19-numbering` |
| [[T20 Outline snapping]] | done | T17 | `task/T20-outline-snapping` |
| [[T21 Cell fill and fill eraser]] | done | T18 | `task/T21-cell-fill` |
| [[T22 Numbering drives grid indices]] | done | T18, T19, T21 | `task/T22-numbered-grids` |
| [[T23 Arrow head proportional to width]] | done | T05 | `fix/arrow-head-scale` |
| [[T14 Performance and release validation]] | done | T12, T13, T15–T23 | `task/T14-release` |
| [[T24 Smooth redraws with large documents]] | done | T14 | `task/T24-layered-redraw` |
| [[T25 Linear-time selection with large documents]] | done | T14 | `task/T25-fast-lookup` |
| [[T26 Pan and zoom without re-rendering]] | done | T24 | `task/T26-gesture-redraw` |
| [[T29 Text blocks]] | done | T14 | `task/T29-text` |
| [[T32 Fast text rendering]] | done | T29 | `task/T32-fast-text` |
| [[T30 Basic TeX formulas]] | done | T29, T32 | `task/T30-tex` |
| [[T33 Math font for formulas]] | done | T30 | `task/T33-math-font` |
| [[T31 Text in rectangles and grid cells]] | done | T29 | `task/T31-shape-text` |
| [[T27 Linux packaging]] | done | T14 | `task/T27-packaging` |
| [[T28 Experimental native Wayland backend]] | done | T27 | `task/T28-wayland` |
| [[T34 Pen tablet input]] | done | T28 | `task/T34-pen-tablet` |
| [[T35 Pen pressure and native Wayland tablets]] | done | T34 | `task/T35-pen-pressure` |
| [[T36 Integrals in formulas]] | done | T33 | `task/T36-integrals` |
| [[T37 Remember the text size]] | done | T29, T31 | `task/T37-text-size-memory` |
| [[T38 Left-hand tool keys]] | done | T11, T12 | `task/T38-left-hand-keys` |
| [[T39 Accents in formulas]] | done | T36 | `task/T39-accents` |
| [[T40 Pen pressure on native Wayland]] | done | T35 | `fix/wayland-pen-pressure` |
| [[T41 Integral superscript at the advance]] | done | T36 | `fix/integral-superscript` |
| [[T42 Join pen tip bounces]] | done | T35 | `fix/pen-tip-bounce` |
| [[T43 Adopt clippy 1.99 lints]] | done | — | `chore/clippy-1-99` |
| [[T44 One keyboard per Wayland seat]] | done | T28 | `fix/wayland-seat-devices` |
| [[T45 Doubled keys after a long idle on Wayland]] | done | T44 | `fix/wayland-idle-doubled-keys` |
| [[T46 Native Wayland by default and Xwayland deprecated]] | done | T28, T44, T45 | `task/T46-wayland-default` |
| [[T47 Always-on diagnostics log]] | done | — | `task/T47-diagnostics-log` |
| [[T48 Clean exit on Ctrl-C]] | done | T47 | `fix/clean-exit-on-sigint` |
| [[T49 Long-running stability]] | done | T47 | `fix/long-running-stability` |
| [[T51 Build-time themes]] | done | — | `task/T51-build-time-themes` |
| [[T52 Text in shapes by default]] | done | — | `task/T52-shape-text-default` |
| [[T54 Math alphabet styles]] | done | — | `task/T54-math-alphabets` |
| [[T55 Math font subset for alphabets and symbols]] | done | T54 | `task/T55-math-font-subset` |
| [[T56 Bold math]] | done | T54, T55 | `task/T56-bold-math` |
| [[T57 Blackboard bold]] | done | T54, T55 | `task/T57-blackboard-bold` |
| [[T58 Fraktur letters]] | done | T54, T55 | `task/T58-fraktur` |
| [[T59 Script letters]] | done | T54, T55 | `task/T59-script-letters` |
| [[T60 Sans-serif and typewriter letters]] | done | T54, T55 | `task/T60-sans-and-mono` |
| [[T61 Hebrew letters]] | done | — | `task/T61-hebrew-letters` |
| [[T62 Complete Greek letters]] | done | T54 | `task/T62-complete-greek` |
| [[T63 Trigonometric and hyperbolic functions]] | done | — | `task/T63-trig-functions` |
| [[T64 Named operators with limits]] | done | T63 | `task/T64-named-operators` |
| [[T65 Set and logic symbols]] | done | T55 | `task/T65-set-logic-symbols` |
| [[T66 Big set and logic operators]] | done | T55, T65 | `task/T66-big-operators` |
| [[T67 Algebra and group theory symbols]] | done | T55, T64 | `task/T67-algebra-symbols` |
| [[T68 Category theory symbols]] | done | T55, T67 | `task/T68-category-symbols` |
| [[T69 Classical physics notation]] | done | T55, T56 | `task/T69-classical-physics` |
| [[T70 Quantum notation]] | done | T69 | `task/T70-quantum-notation` |
| [[T71 Combinatorics notation]] | done | T55 | `task/T71-combinatorics` |
| [[T72 Feature proposal]] | todo | — | `docs/T72-feature-proposal` |
| [[T73 Continuous integration on GitHub]] | done | — | `task/T73-github-ci` |
| [[T74 Editor snapshots for tests]] | done | — | `task/T74-editor-snapshots` |
| [[T75 End-to-end test suite]] | done | T73, T74 | `task/T75-e2e-tests` |
| [[T76 Remembered text size past the ladder]] | done | T74 | `fix/remembered-text-size` |
| [[T77 Test sessions that leave the machine usable]] | done | — | `task/T77-gate-resource-limits` |
| [[T78 Formula ink outside its text box]] | todo | T74 | `fix/formula-ink-outside-box` |
| [[T79 Wayland keyboard without a keymap]] | todo | T75 | `fix/wayland-no-keymap` |
| [[T80 Mouse cursor in end-to-end screenshots]] | done | T75 | `fix/e2e-cursor-in-screenshots` |
| [[T53 Release 1.5.0]] | todo | T46–T49, T51, T52, T54–T80 | `chore/release-1.5.0` |
| [[T50 Version 2.0.0 refactor]] | todo | T46–T49, T51–T72 | `task/T50-v2-refactor` |
