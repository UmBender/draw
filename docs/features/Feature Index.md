---
tags: [moc, feature]
---

# Feature Index

One note per user-visible feature, created by the task that implements it
(template: `templates/Feature.md`). Rows are appended at integration time.

| Feature | Task | Tutorial | Status |
|---------|------|----------|--------|
| [[Infinite canvas, pan and zoom]] | [[T03 Camera]], [[T08 Editor core and input model]], [[T106 Fading laser pointer]], [[T110 Minimap]], [[T116 Pages]] | [[03 Cameras - world space vs screen space]], [[08 An editor as an input-driven state machine]] | done |
| [[Anti-tremor strokes]] | [[T04 Stroke smoothing]], [[T09 Creation tools]] | [[04 Taming shaky input]], [[09 Building drawing tools]] | done |
| [[Shapes]] | [[T05 Shape model]], [[T07 Renderer]], [[T09 Creation tools]], [[T93 Coordinate plane from points]], [[T95 Labels on plotted points]] | [[05 Modelling shapes and hit-testing]], [[07 Rendering with macroquad and culling]], [[09 Building drawing tools]] | done |
| [[Undo and redo]] | [[T06 Document and history]], [[T08 Editor core and input model]], [[T11 Keymap and macros]], [[T49 Long-running stability]] | [[06 Undo and redo with a transaction log]] | done |
| [[Drawing tools]] | [[T09 Creation tools]], [[T109 Shape recognition]] | [[09 Building drawing tools]] | done |
| [[Editing tools]] | [[T10 Editing tools]], [[T91 Resize handles]], [[T100 Z-order keys]], [[T101 Align and distribute]], [[T107 Lock shapes]], [[T115 Copy the selection as an image]] | [[10 Selection, clipboard and fill]] | done |
| [[Shortcuts and macros]] | [[T11 Keymap and macros]], [[T38 Left-hand tool keys]], [[T111 Command palette]], [[T113 Shortcut sheet]] | [[11 Keymaps and gesture macros]], [[38 Laying out shortcuts for one hand]] | done |
| [[Toolbar]] | [[T12 App shell and toolbar]] | [[12 The app loop, idle redraw and UI]] | done |
| [[Theme and palette]] | [[T02 Palette and theme tokens]] | [[02 Palettes and design tokens]] | done |
| [[Themes]] | [[T51 Build-time themes]] | [[51 Keeping every colour in one file]] | done |
| [[Anti-aliasing]] | [[T15 Anti-aliasing]] | [[15 Anti-aliasing and multisampling]] | done |
| [[Snapping]] | [[T17 Snapping]], [[T20 Outline snapping]] | [[17 Snapping and alignment guides]], [[20 Snapping to outlines]] | done |
| [[Grid tool]] | [[T16 Shape model v2 and helper skeleton]], [[T18 Grid tool]], [[T22 Numbering drives grid indices]], [[T84 Grid from text]], [[T85 Resize a committed grid]] | [[18 A grid tool with live parameters]] | done |
| [[Numbered nodes]] | [[T16 Shape model v2 and helper skeleton]], [[T19 Auto-numbering]], [[T22 Numbering drives grid indices]], [[T86 Zero-based numbering]], [[T87 Graph or tree from an edge list]], [[T96 Keywords for graph input]], [[T97 Spaced edge pairs]], [[T98 Math names and weights in graphs]], [[T102 Graph input follow-ups]], [[T104 Segment tree drawing]], [[T119 Forest layout in layers]], [[T120 Weight label on diagonal edges]], [[T121 Self-loops and opposite edges in graphs]] | [[19 Auto-numbering and undoable counters]], [[22 One toggle, two meanings]] | done |
| [[Cell fill]] | [[T21 Cell fill and fill eraser]], [[T88 Bucket drag across cells]] | [[21 Per-cell fills and a two-mode eraser]] | done |
| [[Text blocks]] | [[T29 Text blocks]], [[T37 Remember the text size]], [[T83 Paste text from the clipboard]] | [[29 Typing on a canvas without losing the shortcuts]], [[37 Settings that follow what you last did]] | done |
| [[TeX formulas]] | [[T30 Basic TeX formulas]], [[T33 Math font for formulas]], [[T36 Integrals in formulas]], [[T39 Accents in formulas]], [[T41 Integral superscript at the advance]], [[T54 Math alphabet styles]], [[T61 Hebrew letters]], [[T62 Complete Greek letters]], [[T63 Trigonometric and hyperbolic functions]], [[T64 Named operators with limits]], [[T55 Math font subset for alphabets and symbols]], [[T56 Bold math]], [[T57 Blackboard bold]], [[T58 Fraktur letters]], [[T59 Script letters]], [[T60 Sans-serif and typewriter letters]], [[T65 Set and logic symbols]], [[T66 Big set and logic operators]], [[T67 Algebra and group theory symbols]], [[T68 Category theory symbols]], [[T69 Classical physics notation]], [[T70 Quantum notation]], [[T71 Combinatorics notation]], [[T89 Growing brackets]], [[T90 Matrices and cases]], [[T99 Spacing commands]], [[T103 Labelled extensible arrows]], [[T105 Display mode]], [[T112 User macros]] | [[30 Parsing and laying out a small TeX subset]], [[33 Typesetting with a proportional math font]], [[36 Operators whose limits sit beside them]], [[39 Accents that sit on their letters]], [[54 Math alphabets as a mapping table]] | done |
| [[Text in shapes]] | [[T31 Text in rectangles and grid cells]], [[T52 Text in shapes by default]] | [[31 Prototyping an interaction behind a switch]] | done |
| [[Linux packages]] | [[T27 Linux packaging]], [[T82 Raise the binary size limit]] | [[27 Shipping a binary to other Linux machines]] | done |
| [[Native Wayland]] | [[T28 Experimental native Wayland backend]], [[T44 One keyboard per Wayland seat]], [[T46 Native Wayland by default and Xwayland deprecated]], [[T79 Wayland keyboard without a keymap]] | [[28 Choosing a window backend at startup]] | done |
| [[Pen tablets]] | [[T34 Pen tablet input]], [[T35 Pen pressure and native Wayland tablets]], [[T40 Pen pressure on native Wayland]], [[T42 Join pen tip bounces]] | [[34 Telling a tap from a drag]], [[35 Reading a pen through a patched window library]] | done |
| [[Diagnostics log]] | [[T47 Always-on diagnostics log]] | [[47 A flight recorder for rare bugs]] | done |
| [[Clean exit on signals]] | [[T48 Clean exit on Ctrl-C]] | [[48 Signals, self-pipes and async-signal safety]] | done |
| [[Editor snapshots]] | [[T74 Editor snapshots for tests]], [[T108 Save and load]] | — | done |
| [[End-to-end tests]] | [[T75 End-to-end test suite]], [[T79 Wayland keyboard without a keymap]], [[T80 Mouse cursor in end-to-end screenshots]] | — | done |
| [[Contest cheat sheet]] | [[T14 Performance and release validation]] | [[14 Profiling and shipping a release build]] | done |
| [[Crash recovery]] | [[T94 Autosave and crash recovery]] | — | done |
| [[PNG export]] | [[T92 Export the view to PNG]], [[T114 Export to SVG]] | — | done |
