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
| 12 | [[ADR-0012 Fixed palette and theme tokens]] | accepted | decision | 0004 | amended by 0015 |
| 13 | [[ADR-0013 World-space widths and zoom limits]] | accepted | decision | 0004 | — |
| 14 | [[ADR-0014 Minimal dependencies and no unsafe]] | accepted | decision | 0002, 0003 | — |
| 15 | [[ADR-0015 Kanagawa Dragon theme]] | accepted | decision | 0012 | amends 0012 |
| 16 | [[ADR-T06-1 Self-checking edits and rollback atomicity]] | accepted | decision | 0005 | — |
| 17 | [[ADR-T07-1 Screen-space tessellation in the renderer]] | accepted | decision | 0002, 0006, 0013 | amended by T24-4 |
| 18 | [[ADR-T08-1 Tool context and gesture overlay]] | accepted | decision | 0003, 0005 | — |
| 19 | [[ADR-T11-1 Exact modifier matching]] | accepted | decision | 0003 | — |
| 20 | [[ADR-T12-1 Blocking event loop with cached frame]] | accepted | decision | 0006, 0002 | settles the open choice in 0006; amended by T24-1 |
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
| 35 | [[ADR-T14-1 Performance budgets]] | accepted | decision | 0006, 0007, T17-1, T20-1 | amended by T24-5 |
| 36 | [[ADR-T14-2 Low smoothing by default]] | accepted | decision | 0007 | amends 0007 |
| 37 | [[ADR-T24-1 Document layer keyed by a document revision]] | accepted | decision | 0006, T12-1, T15-1, T08-1 | amends T12-1; amended by T24-2, T24-3 |
| 38 | [[ADR-T24-2 Overlay drawn straight to the window]] | accepted | decision | T24-1, T12-1 | amends T24-1, T15-1 |
| 39 | [[ADR-T24-3 Batched meshes and append-only document updates]] | accepted | decision | T07-1, T24-1, 0014 | amends T24-1 |
| 40 | [[ADR-T24-4 Strokes as one strip with sparse round joins]] | accepted | decision | T07-1, T24-3 | amends T07-1 |
| 41 | [[ADR-T24-5 Redraw budgets]] | accepted | decision | T24-1, T24-2, T24-3, T24-4 | amends T14-1 |
