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
| 4 | [[ADR-0004 Vector object model]] | accepted | decision | 0003 | — |
| 5 | [[ADR-0005 Undo via transaction log]] | accepted | decision | 0004 | — |
| 6 | [[ADR-0006 Redraw on demand]] | accepted | decision | 0002, 0003 | — |
| 7 | [[ADR-0007 Anti-tremor pipeline]] | accepted | decision | 0004 | — |
| 8 | [[ADR-0008 Testing strategy SDD and TDD]] | accepted | decision | 0003 | — |
| 9 | [[ADR-0009 Fuzzing with proptest]] | accepted | decision | 0003, 0008 | — |
| 10 | [[ADR-0010 Clippy lint policy]] | accepted | decision | 0008 | — |
| 11 | [[ADR-0011 Git workflow]] | accepted | decision | 0001, 0008 | — |
| 12 | [[ADR-0012 Fixed palette and theme tokens]] | accepted | decision | 0004 | amended by 0015 |
| 13 | [[ADR-0013 World-space widths and zoom limits]] | accepted | decision | 0004 | — |
| 14 | [[ADR-0014 Minimal dependencies and no unsafe]] | accepted | decision | 0002, 0003 | — |
| 15 | [[ADR-0015 Kanagawa Dragon theme]] | accepted | decision | 0012 | amends 0012 |
| 16 | [[ADR-T06-1 Self-checking edits and rollback atomicity]] | accepted | decision | 0005 | — |
| 17 | [[ADR-T07-1 Screen-space tessellation in the renderer]] | accepted | decision | 0002, 0006, 0013 | — |
| 18 | [[ADR-T08-1 Tool context and gesture overlay]] | accepted | decision | 0003, 0005 | — |
| 19 | [[ADR-T11-1 Exact modifier matching]] | accepted | decision | 0003 | — |
| 20 | [[ADR-T12-1 Blocking event loop with cached frame]] | accepted | decision | 0006, 0002 | settles the open choice in 0006 |
