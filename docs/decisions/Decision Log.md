---
tags: [moc, adr]
---

# Decision Log

The ordered chain of architecture decisions. **Append-only**: new rows go at the
bottom, at integration time. Rules: [[Decision Protocol]].

| # | ADR | Status | Kind | Builds on | Changes |
|---|-----|--------|------|-----------|---------|
| 1 | [[ADR-0001 Record decisions as an append-only log]] | accepted | decision | — | — |
| 2 | [[ADR-0002 Rust and macroquad]] | accepted | decision | 0001 | — |
| 3 | [[ADR-0003 Headless core and thin shell]] | accepted | decision | 0002 | — |
| 4 | [[ADR-0004 Vector object model]] | accepted | decision | 0003 | amended by T16-1 |
| 5 | [[ADR-0005 Undo via transaction log]] | accepted | decision | 0004 | — |
| 6 | [[ADR-0006 Redraw on demand]] | accepted | decision | 0002, 0003 | — |
| 7 | [[ADR-0007 Anti-tremor pipeline]] | accepted | decision | 0004 | amended by T14-2 |
| 8 | [[ADR-0008 Testing strategy SDD and TDD]] | accepted | decision | 0003 | — |
| 9 | [[ADR-0009 Fuzzing with proptest]] | accepted | decision | 0003, 0008 | — |
| 10 | [[ADR-0010 Clippy lint policy]] | accepted | decision | 0008 | — |
| 11 | [[ADR-0011 Git workflow]] | accepted | decision | 0001, 0008 | — |
| 12 | [[ADR-0012 Fixed palette and theme tokens]] | accepted | decision | 0004 | amended by 0015, T51-1 |
| 13 | [[ADR-0013 World-space widths and zoom limits]] | accepted | decision | 0004 | — |
| 14 | [[ADR-0014 Minimal dependencies and no unsafe]] | accepted | decision | 0002, 0003 | amended by T32-1, T35-1 |
| 15 | [[ADR-0015 Kanagawa Dragon theme]] | accepted | decision | 0012 | amends 0012, amended by T51-1 |
| 16 | [[ADR-T06-1 Self-checking edits and rollback atomicity]] | accepted | decision | 0005 | — |
| 17 | [[ADR-T07-1 Screen-space tessellation in the renderer]] | accepted | decision | 0002, 0006, 0013 | amended by T24-4 |
| 18 | [[ADR-T08-1 Tool context and gesture overlay]] | accepted | decision | 0003, 0005 | — |
| 19 | [[ADR-T11-1 Exact modifier matching]] | accepted | decision | 0003 | — |
| 20 | [[ADR-T12-1 Blocking event loop with cached frame]] | accepted | decision | 0006, 0002 | settles the open choice in 0006; amended by T24-1, T26-1 |
| 21 | [[ADR-T15-1 MSAA on the cached frame]] | accepted | decision | 0006, T12-1, 0014 | amended by T24-2 |
| 22 | [[ADR-T16-1 Grid shape and shape labels]] | accepted | decision | 0013, T07-1 | amends 0004 |
| 23 | [[ADR-T16-2 Helper key bindings]] | accepted | decision | T11-1 | — |
| 24 | [[ADR-T16-3 Helper settings and hooks]] | accepted | decision | T08-1, T16-1 | — |
| 25 | [[ADR-T17-1 Snapping order and tolerances]] | accepted | decision | T16-3, T16-1, 0013 | amended by T20-1 |
| 26 | [[ADR-T19-1 Numbering counter and undo]] | accepted | decision | T16-1, T16-3, T08-1 | — |
| 27 | [[ADR-T18-1 Grid drag reads live dims and snaps as a box]] | accepted | decision | T08-1, T16-1, T16-2, T16-3, T17-1 | — |
| 28 | [[ADR-T18-2 Grid size flyout in the toolbar]] | accepted | decision | T16-2, T16-3, T18-1 | amended by T22-1 |
| 29 | [[ADR-T18-3 Grid axis indices]] | accepted | decision | T16-1, T16-2, T16-3, T18-1, T18-2 | amended by T22-1 |
| 30 | [[ADR-T21-1 Cell fills and a two-mode eraser]] | accepted | decision | 0004, T08-1, T16-1, T18-3 | amended by T21-2 |
| 31 | [[ADR-T21-2 Eraser mode chosen on press]] | accepted | decision | T21-1 | amends T21-1 |
| 32 | [[ADR-T22-1 Numbering toggle drives grid indices]] | accepted | decision | T18-3, T19-1, T16-2 | amends T18-3, T18-2 |
| 33 | [[ADR-0016 Arrow head proportional to width]] | accepted | decision | 0004, 0013 | — |
| 34 | [[ADR-T20-1 Outline snapping]] | accepted | decision | T17-1, 0013 | amends T17-1 |
| 35 | [[ADR-T14-1 Performance budgets]] | accepted | decision | 0006, 0007, T17-1, T20-1 | amended by T24-5, T25-2 |
| 36 | [[ADR-T14-2 Low smoothing by default]] | accepted | decision | 0007 | amends 0007 |
| 37 | [[ADR-T24-1 Document layer keyed by a document revision]] | accepted | decision | 0006, T12-1, T15-1, T08-1 | amends T12-1; amended by T24-2, T24-3, T26-1 |
| 38 | [[ADR-T24-2 Overlay drawn straight to the window]] | accepted | decision | T24-1, T12-1 | amends T24-1, T15-1 |
| 39 | [[ADR-T24-3 Batched meshes and append-only document updates]] | accepted | decision | T07-1, T24-1, 0014 | amends T24-1; amended by T32-2 |
| 40 | [[ADR-T24-4 Strokes as one strip with sparse round joins]] | accepted | decision | T07-1, T24-3 | amends T07-1; amended by T35-2 |
| 41 | [[ADR-T24-5 Redraw budgets]] | accepted | decision | T24-1, T24-2, T24-3, T24-4 | amends T14-1; amended by T26-2 |
| 42 | [[ADR-T25-1 Lazy position index and batched edit runs]] | accepted | decision | 0004, 0005, T06-1 | — |
| 43 | [[ADR-T25-2 Id hasher and selection-scale budgets]] | accepted | decision | T25-1, T14-1, 0014 | amends T14-1 |
| 44 | [[ADR-T26-1 Reuse the document layer during pan and zoom]] | accepted | decision | T12-1, T15-1, T24-1, T24-2 | amends T12-1, T24-1; amended by T26-3 |
| 45 | [[ADR-T26-2 Gesture frame budget]] | proposed | decision | T26-1 | amends T24-5 |
| 46 | [[ADR-T26-3 Reuse the layer only when re-rendering is slow]] | accepted | decision | T26-1 | amends T26-1 |
| 47 | [[ADR-T29-1 Text shape with monospaced layout and an embedded font]] | accepted | decision | 0003, 0013, 0014, T07-1 | amends 0004, T16-1 |
| 48 | [[ADR-T29-2 Text edit mode]] | accepted | decision | 0005, T08-1, T11-1, T24-1, T29-1 | amended by T29-3 |
| 49 | [[ADR-T29-3 Resize text while typing]] | accepted | decision | T29-1 | amends T29-2 |
| 50 | [[ADR-T32-1 Own glyph atlas rasterized with fontdue]] | accepted | decision | 0014, T24-3, T29-1 | amends 0014 |
| 51 | [[ADR-T32-2 Shapes and glyphs share one textured batch]] | accepted | decision | T24-3, T32-1 | amends T24-3 |
| 52 | [[ADR-T30-1 TeX subset, box layout and metrics]] | accepted | decision | 0003, 0014, T29-1, T29-2, T32-1, T32-2 | — |
| 53 | [[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]] | accepted | decision | 0003, 0014, T29-1, T32-1, T32-2 | amends T30-1, amended by T55-1 |
| 54 | [[ADR-T31-1 Shape text prototype behind a switch]] | superseded | decision | T16-1, T16-3, T19-1, T21-1, T21-2, T29-1, T29-2, T29-3, T30-1 | amends T16-1, T21-2, T29-2, superseded by T52-1 |
| 55 | [[ADR-T27-1 Build in an AlmaLinux 8 container for a glibc 2.28 floor]] | accepted | decision | 0002, 0014 | — |
| 56 | [[ADR-T27-2 Package formats from one staging tree and one library list]] | accepted | decision | 0002, 0014, T12-1, T27-1 | — |
| 57 | [[ADR-T27-3 Crash report from a panic hook and split debug symbols]] | accepted | decision | 0002, 0014 | — |
| 58 | [[ADR-T28-1 Opt-in native Wayland backend]] | accepted | decision | T12-1, T26-1, T27-2 | — |
| 59 | [[ADR-T34-1 Tap slop for pen tablets detected at start-up]] | accepted | decision | 0003, T28-1 | — |
| 60 | [[ADR-T35-1 Vendored miniquad with a pen tablet patch]] | accepted | decision | T12-1, T28-1, T34-1 | amends 0014 |
| 61 | [[ADR-T35-2 Pressure as per-point width factors on pen strokes]] | accepted | decision | 0007, 0013, T35-1 | amends T24-4 |
| 62 | [[ADR-T38-1 Left-hand keys for pen, line and smart snap]] | accepted | decision | T11-1 | — |
| 63 | [[ADR-T36-1 Integral class with scripts beside the sign]] | accepted | decision | T30-1, T33-1 | — |
| 64 | [[ADR-T37-1 Remember the last text size]] | accepted | decision | 0013, T29-2, T31-1 | amends T29-3 |
| 65 | [[ADR-T39-1 Accents from spacing glyphs with a slant skew]] | accepted | decision | T30-1, T33-1 | — |
| 66 | [[ADR-T40-1 Tablet stream wins over copied wl_pointer input]] | accepted | decision | T35-2 | amends T35-1 |
| 67 | [[ADR-T41-1 Integral superscript at the advance]] | accepted | decision | T33-1 | amends T36-1 |
| 68 | [[ADR-T42-1 Pen tip bounces join the stroke]] | accepted | decision | 0003, T34-1, T35-1 | — |
| 69 | [[ADR-T44-1 Wayland seat devices follow the announced capabilities]] | accepted | decision | T28-1 | amends T35-1 |
| 70 | [[ADR-T46-1 Native Wayland by default and X11 deprecated]] | accepted | decision | T27-2, T44-1 | amends T28-1 |
| 71 | [[ADR-T47-1 Flight recorder and capped session log]] | accepted | decision | 0014, T27-3, T35-1, T44-1, T24-5 | — |
| 72 | [[ADR-T48-1 Signal handling in the vendored miniquad]] | accepted | decision | 0014, T35-1, T27-3, T47-1, T12-1 | — |
| 73 | [[ADR-T49-1 Undo history capped by bytes]] | accepted | decision | 0005, T24-1 | amends 0005 |
| 74 | [[ADR-T73-1 Track the latest stable Rust]] | accepted | decision | T27-1, 0010 | — |
| 75 | [[ADR-T74-1 Line-based editor snapshot format]] | accepted | decision | 0003, 0014, 0009, T49-1 | — |
| 76 | [[ADR-T64-1 Operatorname with a star parsed as one command]] | accepted | decision | T30-1 | — |
| 77 | [[ADR-T54-1 Math alphabets as a table of per-kind mappings]] | accepted | decision | T30-1 | amends T33-1, amended by T56-1 |
| 78 | [[ADR-T52-1 Text in shapes always on, numbered shapes hold no text]] | accepted | decision | T16-1, T16-3, T19-1, T74-1 | supersedes T31-1 |
| 79 | [[ADR-T51-1 One theme file with every colour]] | accepted | decision | 0012, 0015 | amends 0012, 0015 |
| 80 | [[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]] | accepted | decision | T14-1, T32-1 | amends T33-1 |
| 81 | [[ADR-T56-1 Accent skew follows a slant flag per math alphabet]] | accepted | decision | T39-1 | amends T54-1 |
| 82 | [[ADR-T57-1 One double-struck alphabet for mathbb and mathds]] | accepted | decision | T54-1, T56-1 | — |
| 83 | [[ADR-T59-1 One script alphabet for mathscr and mathcal]] | accepted | decision | T54-1, T55-1 | — |
| 84 | [[ADR-T60-1 Bold sans-serif inside a bold alphabet]] | accepted | decision | T54-1, T56-1 | — |
| 85 | [[ADR-T65-1 Wide relations padded with a thick space]] | accepted | decision | T30-1 | — |
| 86 | [[ADR-T71-1 Infix fraction commands split their group]] | accepted | decision | T30-1 | — |
| 87 | [[ADR-T69-1 Physics shortcuts expand to their long forms]] | accepted | decision | T30-1, T39-1 | — |
| 88 | [[ADR-T77-1 Test gates run in capped low-priority scopes]] | accepted | decision | — | — |
| 89 | [[ADR-T76-1 Remembered text size stays on the ladder]] | accepted | decision | T29-3, T74-1 | amends T37-1 |
| 90 | [[ADR-T75-1 Headless sway session for end-to-end tests]] | accepted | decision | T74-1, T77-1, T46-1 | — |
| 91 | [[ADR-T75-2 Wheel notches through the virtual pointer]] | accepted | decision | — | amends T75-1 |
| 92 | [[ADR-T80-1 Screenshots ignore the box under the pointer]] | accepted | decision | — | amends T75-1 |
| 93 | [[ADR-T79-1 Ignore an unusable Wayland keymap]] | accepted | decision | T35-1, T47-1 | — |
| 94 | [[ADR-T82-1 Shipped binary limit of 20 MiB]] | accepted | decision | T27-3 | amends T29-1 |
| 95 | [[ADR-T83-1 Paste text through an input event]] | accepted | decision | T29-2, T35-1, T75-1, T12-1 | amends T29-2 |
| 96 | [[ADR-T84-1 Grid from text keys and table rules]] | accepted | decision | T83-1, T11-1, T16-1, T22-1, T31-1, T47-1 | — |
| 97 | [[ADR-T85-1 Arrows resize the one selected grid]] | accepted | decision | T16-1, T16-2, T18-3, T21-1, T31-1, 0005 | amends T16-2 |
| 98 | [[ADR-T86-1 Zero-based numbering switch]] | accepted | decision | T16-2, T16-3, T19-1 | amends T52-1, amends T74-1 |
| 99 | [[ADR-T87-1 Graph from an edge list key and layout rules]] | accepted | decision | T84-1, T11-1, T16-1, T19-1, T20-1, T31-1, T47-1 | — |
| 100 | [[ADR-T88-1 Bucket drag previews cells and commits on release]] | accepted | decision | T21-1, T21-2, T74-1 | — |
| 101 | [[ADR-T89-1 Left and right brackets centred on the axis]] | accepted | decision | T30-1, T33-1 | — |
| 102 | [[ADR-T90-1 Environments as one table node]] | accepted | decision | T30-1, T89-1 | — |
| 103 | [[ADR-T91-1 Resize handles and scale rules]] | accepted | decision | 0004, 0005, T08-1, T24-2 | — |
| 104 | [[ADR-T91-2 Scaled text stays within the largest text size]] | accepted | decision | T74-1 | amends T91-1 |
| 105 | [[ADR-T94-1 Autosave captured on input and written off the frame]] | accepted | decision | T12-1, T24-5, T47-1, T74-1 | — |
| 106 | [[ADR-T92-1 The png crate as a direct dependency]] | accepted | decision | 0014, T32-1 | amends 0014 |
| 107 | [[ADR-T92-2 Ctrl+E reads back the document layer]] | accepted | decision | 0003, T11-1, T15-1, T24-1, T24-2, T26-1, T29-2, T32-1, T92-1 | — |
| 108 | [[ADR-T93-1 Coordinate plane key and plotting rules]] | accepted | decision | T87-1, T84-1, T11-1, T47-1 | — |
| 109 | [[ADR-T95-1 Plotted point labels]] | accepted | decision | T93-1 | amends T93-1 |
| 110 | [[ADR-T96-1 Keywords for graph input]] | accepted | decision | T87-1 | amends T87-1 |
| 111 | [[ADR-T97-1 Spaced edge pairs]] | accepted | decision | T87-1, T96-1 | amends T87-1 |
| 112 | [[ADR-T98-1 Math names and weights in graphs]] | accepted | decision | T87-1, T96-1, T97-1, T52-1 | amends T87-1 |
| 113 | [[ADR-T99-1 Spacing commands as kerns outside the atom list]] | accepted | decision | T30-1, T36-1 | — |
| 114 | [[ADR-T100-1 Z-order keys and restack transactions]] | accepted | decision | 0005, T06-1, T11-1, T29-3 | — |
| 115 | [[ADR-T101-1 Align and distribute keys]] | accepted | decision | T11-1, T29-2, T85-1, T91-1, T100-1 | — |
| 116 | [[ADR-T101-2 Rows and columns without overlap]] | accepted | decision | T101-1 | amends T101-1 |
| 117 | [[ADR-T101-3 Compact rows and columns]] | accepted | decision | T101-2 | amends T101-2 |
| 118 | [[ADR-T102-1 Weights after pairs, root 0 and weighted keywords]] | accepted | decision | T96-1, T97-1, T98-1 | amends T96-1, T97-1 |
| 119 | [[ADR-T103-1 Extensible arrows from the arrow glyph and a ruled shaft]] | accepted | decision | T30-1, T33-1 | — |
| 120 | [[ADR-T104-1 Segment tree keyword]] | accepted | decision | T87-1, T96-1, T98-1, T86-1 | amends T96-1 |
| 121 | [[ADR-T104-2 Segment tree nodes as boxes]] | accepted | decision | T104-1, T87-1 | amends T104-1 |
| 122 | [[ADR-T105-1 Display style as a style field and display segments]] | accepted | decision | T30-1, T54-1, T71-1 | — |
| 123 | [[ADR-T106-1 Laser pointer in front of the editor]] | accepted | decision | 0006, T24-2, T24-5, T74-1 | — |
| 124 | [[ADR-T106-2 Laser key, fade and look]] | accepted | decision | T106-1, T11-1 | — |
| 125 | [[ADR-T107-1 Lock flag kept by the document]] | accepted | decision | T06-1, T74-1 | — |
| 126 | [[ADR-T107-2 Ctrl+K locks and Ctrl+Shift+K unlocks]] | accepted | decision | T107-1 | — |
| 127 | [[ADR-T109-1 Shape recognition on Z]] | accepted | decision | 0005, T11-1, T29-2, T74-1 | — |
| 128 | [[ADR-T110-1 Minimap geometry, routing and overview cache]] | accepted | decision | 0003, 0006, T08-1, T24-1, T24-2, T74-1 | — |
| 129 | [[ADR-T111-1 Command palette as a modal editor state]] | accepted | decision | T11-1, 0006 | — |
| 130 | [[ADR-T119-1 Forests drawn in layers]] | accepted | decision | T87-1, T104-2 | amends T87-1 |
| 131 | [[ADR-T121-1 Self-loops and lanes for edges between the same nodes]] | accepted | decision | T87-1, T119-1 | amends T87-1 |
| 132 | [[ADR-T108-1 The draw file format and its promise]] | accepted | decision | T74-1, T116-1 | — |
| 133 | [[ADR-T112-1 User macros as source expansion]] | accepted | decision | T69-1, T105-1 | — |
| 134 | [[ADR-T113-1 Shortcut sheet as a second modal overlay]] | accepted | decision | T111-1 | — |
| 135 | [[ADR-T114-1 The ttf-parser crate as a direct dependency]] | accepted | decision | 0014, T92-1 | amends 0014 |
| 136 | [[ADR-T114-2 SVG export is a pure writer over glyph outlines]] | accepted | decision | T92-2, T30-1 | — |
| 137 | [[ADR-T115-1 Selection image on the clipboard]] | accepted | decision | T35-1, T92-1 | — |
| 138 | [[ADR-T116-1 Nine pages swapped into the editor]] | accepted | decision | — | — |
| 139 | [[ADR-T123-1 Menus use the bundled font on whole pixels]] | accepted | decision | T29-1 | — |
| 140 | [[ADR-T124-1 Ten pages and a switcher under undo and redo]] | accepted | decision | T116-1, T123-1 | amends T116-1 |
| 141 | [[ADR-T117-1 Scored shape recognition with looser limits]] | accepted | decision | T109-1 | amends T109-1 |
| 142 | [[ADR-T125-1 Six pages, one button per row]] | accepted | decision | T124-1 | amends T124-1 |
| 143 | [[ADR-T126-1 Menus drawn as translucent rounded islands]] | accepted | decision | T51-1, T123-1 | — |
| 144 | [[ADR-T127-1 The 2.0 architecture]] | accepted | decision | 0003 | — |
| 145 | [[ADR-T127-2 The unsafe policy]] | accepted | decision | T127-1 | amends 0014 |
| 146 | [[ADR-T127-3 The dependency list]] | accepted | decision | T127-1, T32-1, T92-1, T114-1 | amends 0014 |
| 147 | [[ADR-T127-4 An own backend replaces miniquad and macroquad]] | accepted | decision | T127-1, T127-2 | supersedes 0002 |
| 148 | [[ADR-T127-5 OpenGL ES 3.0 first, Vulkan optional]] | accepted | decision | T127-1, T127-4 | — |
| 149 | [[ADR-T128-1 X11 stays supported]] | accepted | decision | T28-1, T44-1 | reverts T46-1 |
| 150 | [[ADR-T131-1 No dyn outside an allow-list]] | accepted | decision | 0010, 0014 | — |
| 151 | [[ADR-T132-1 Workspace of five crates with unsafe levels]] | accepted | decision | T127-1, T127-2, T127-3 | — |
| 152 | [[ADR-T152-1 Keep DRAW_TABLET as a debugging variable]] | superseded | decision | — | — |
| 153 | [[ADR-T133-1 Typed layer added beside the public API]] | accepted | decision | — | — |
| 154 | [[ADR-T134-1 The automaton wraps the editor behind a routing table]] | accepted | decision | — | — |
| 155 | [[ADR-T135-1 Regions live in the machine module]] | accepted | decision | — | — |
| 156 | [[ADR-T136-1 One gesture variant per tool in the machine]] | accepted | decision | — | — |
| 157 | [[ADR-T137-1 One effect queue and time as a parameter]] | accepted | decision | — | — |
| 158 | [[ADR-T153-1 Text and select are sub-automata of the machine]] | accepted | decision | T134-1, T136-1 | amends T136-1 |
| 159 | [[ADR-T154-1 Every DRAW_ variable the binary reads becomes a flag]] | accepted | decision | T152-1 | supersedes T152-1 |
| 160 | [[ADR-T140-1 The Gpu and Platform traits]] | accepted | decision | T127-5, T131-1, T132-1 | — |
