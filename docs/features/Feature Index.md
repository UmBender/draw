---
tags: [moc, feature]
---

# Feature Index

One note per user-visible feature, created by the task that implements it
(template: `templates/Feature.md`). Rows are appended at integration time.

| Feature | Task | Tutorial | Status |
|---------|------|----------|--------|
| [[Infinite canvas, pan and zoom]] | [[T03 Camera]], [[T08 Editor core and input model]] | [[03 Cameras - world space vs screen space]], [[08 An editor as an input-driven state machine]] | done |
| [[Anti-tremor strokes]] | [[T04 Stroke smoothing]], [[T09 Creation tools]] | [[04 Taming shaky input]], [[09 Building drawing tools]] | done |
| [[Shapes]] | [[T05 Shape model]], [[T07 Renderer]], [[T09 Creation tools]] | [[05 Modelling shapes and hit-testing]], [[07 Rendering with macroquad and culling]], [[09 Building drawing tools]] | done |
| [[Undo and redo]] | [[T06 Document and history]], [[T08 Editor core and input model]], [[T11 Keymap and macros]], [[T49 Long-running stability]] | [[06 Undo and redo with a transaction log]] | done |
| [[Drawing tools]] | [[T09 Creation tools]] | [[09 Building drawing tools]] | done |
| [[Editing tools]] | [[T10 Editing tools]] | [[10 Selection, clipboard and fill]] | done |
| [[Shortcuts and macros]] | [[T11 Keymap and macros]], [[T38 Left-hand tool keys]] | [[11 Keymaps and gesture macros]], [[38 Laying out shortcuts for one hand]] | done |
| [[Toolbar]] | [[T12 App shell and toolbar]] | [[12 The app loop, idle redraw and UI]] | done |
| [[Theme and palette]] | [[T02 Palette and theme tokens]] | [[02 Palettes and design tokens]] | done |
| [[Themes]] | [[T51 Build-time themes]] | [[51 Keeping every colour in one file]] | done |
| [[Anti-aliasing]] | [[T15 Anti-aliasing]] | [[15 Anti-aliasing and multisampling]] | done |
| [[Snapping]] | [[T17 Snapping]], [[T20 Outline snapping]] | [[17 Snapping and alignment guides]], [[20 Snapping to outlines]] | done |
| [[Grid tool]] | [[T16 Shape model v2 and helper skeleton]], [[T18 Grid tool]], [[T22 Numbering drives grid indices]] | [[18 A grid tool with live parameters]] | done |
| [[Numbered nodes]] | [[T16 Shape model v2 and helper skeleton]], [[T19 Auto-numbering]], [[T22 Numbering drives grid indices]] | [[19 Auto-numbering and undoable counters]], [[22 One toggle, two meanings]] | done |
| [[Cell fill]] | [[T21 Cell fill and fill eraser]] | [[21 Per-cell fills and a two-mode eraser]] | done |
| [[Text blocks]] | [[T29 Text blocks]], [[T37 Remember the text size]] | [[29 Typing on a canvas without losing the shortcuts]], [[37 Settings that follow what you last did]] | done |
| [[TeX formulas]] | [[T30 Basic TeX formulas]], [[T33 Math font for formulas]], [[T36 Integrals in formulas]], [[T39 Accents in formulas]], [[T41 Integral superscript at the advance]], [[T54 Math alphabet styles]], [[T61 Hebrew letters]], [[T62 Complete Greek letters]], [[T63 Trigonometric and hyperbolic functions]], [[T64 Named operators with limits]], [[T55 Math font subset for alphabets and symbols]], [[T56 Bold math]], [[T57 Blackboard bold]], [[T58 Fraktur letters]], [[T59 Script letters]], [[T60 Sans-serif and typewriter letters]], [[T65 Set and logic symbols]], [[T66 Big set and logic operators]], [[T67 Algebra and group theory symbols]], [[T68 Category theory symbols]], [[T69 Classical physics notation]], [[T70 Quantum notation]], [[T71 Combinatorics notation]] | [[30 Parsing and laying out a small TeX subset]], [[33 Typesetting with a proportional math font]], [[36 Operators whose limits sit beside them]], [[39 Accents that sit on their letters]], [[54 Math alphabets as a mapping table]] | done |
| [[Text in shapes]] | [[T31 Text in rectangles and grid cells]], [[T52 Text in shapes by default]] | [[31 Prototyping an interaction behind a switch]] | done |
| [[Linux packages]] | [[T27 Linux packaging]] | [[27 Shipping a binary to other Linux machines]] | done |
| [[Native Wayland]] | [[T28 Experimental native Wayland backend]], [[T44 One keyboard per Wayland seat]], [[T46 Native Wayland by default and Xwayland deprecated]] | [[28 Choosing a window backend at startup]] | done |
| [[Pen tablets]] | [[T34 Pen tablet input]], [[T35 Pen pressure and native Wayland tablets]], [[T40 Pen pressure on native Wayland]], [[T42 Join pen tip bounces]] | [[34 Telling a tap from a drag]], [[35 Reading a pen through a patched window library]] | done |
| [[Diagnostics log]] | [[T47 Always-on diagnostics log]] | [[47 A flight recorder for rare bugs]] | done |
| [[Clean exit on signals]] | [[T48 Clean exit on Ctrl-C]] | [[48 Signals, self-pipes and async-signal safety]] | done |
| [[Editor snapshots]] | [[T74 Editor snapshots for tests]] | — | done |
| [[End-to-end tests]] | [[T75 End-to-end test suite]], [[T80 Mouse cursor in end-to-end screenshots]] | — | done |
| [[Contest cheat sheet]] | [[T14 Performance and release validation]] | [[14 Profiling and shipping a release build]] | done |
