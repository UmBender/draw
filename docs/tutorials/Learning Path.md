---
tags: [moc, tutorial]
---

# Learning Path

The tutorials, read in order, teach how to build `draw` from scratch. Each task
writes its tutorial (template: `templates/Tutorial.md`) and states which steps
it requires. Planned order follows the task chain:

```mermaid
flowchart LR
  s0[0 Project layout & tooling] --> s1[1 2D geometry]
  s0 --> s2[2 Palettes & theme tokens]
  s1 --> s3[3 Cameras: world vs screen]
  s1 --> s4[4 Smoothing & Douglas–Peucker]
  s1 --> s5[5 Shapes & hit-testing]
  s5 --> s6[6 Undo with transactions]
  s3 --> s7[7 Immediate-mode rendering]
  s6 --> s8[8 Editors as state machines]
  s8 --> s9[9 Drawing tools]
  s8 --> s10[10 Editing tools]
  s8 --> s11[11 Keymaps & macros]
  s7 --> s12[12 The app loop & idle redraw]
  s9 --> s13[13 Property testing & fuzzing]
  s13 --> s14[14 Profiling & release builds]
```

| Step | Tutorial | Task | Status |
|------|----------|------|--------|
| 0 | [[00 Project layout, lints and the TDD loop]] | [[T00 Bootstrap]] | done |
| 1 | [[01 2D vectors, AABBs and point-segment distance]] | [[T01 Geometry primitives]] | done |
| 2 | [[02 Palettes and design tokens]] | [[T02 Palette and theme tokens]] | done |
| 3 | [[03 Cameras - world space vs screen space]] | [[T03 Camera]] | done |
| 4 | [[04 Taming shaky input]] | [[T04 Stroke smoothing]] | done |
| 5 | [[05 Modelling shapes and hit-testing]] | [[T05 Shape model]] | done |
| 6 | [[06 Undo and redo with a transaction log]] | [[T06 Document and history]] | done |
| 7 | [[07 Rendering with macroquad and culling]] | [[T07 Renderer]] | done |
| 8 | [[08 An editor as an input-driven state machine]] | [[T08 Editor core and input model]] | done |
| 9 | *Building drawing tools* | [[T09 Creation tools]] | planned |
| 10 | *Selection, clipboard and fill* | [[T10 Editing tools]] | planned |
| 11 | *Keymaps and gesture macros* | [[T11 Keymap and macros]] | planned |
| 12 | *The app loop: input, idle redraw, UI* | [[T12 App shell and toolbar]] | planned |
| 13 | *Property-based testing and fuzzing with proptest* | [[T13 Fuzz harness]] | planned |
| 14 | *Profiling and shipping a release build* | [[T14 Performance and release validation]] | planned |
