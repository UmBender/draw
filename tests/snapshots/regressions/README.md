# Regression snapshots

One snapshot per fixed bug, saved just before the bug goes wrong
(`docs/process/Workflow.md` → *Bug fixes: regression step*).

1. Reproduce the bug in the app and press `Ctrl+Shift+S` right before
   the step that breaks. The path is printed and logged
   (`~/.local/state/draw/snapshots/snapshot-<UTC>.snap`). From a test,
   write `editor.snapshot().to_text()` instead.
2. Copy it here as `<ID>-<slug>.snap`, e.g. `T80-paste-into-grid.snap`.
   `DRAW_SNAPSHOT=<file> draw` opens it, to check it shows the state.
3. Write `regression_<ID>_<slug>` that loads it with
   `common::snapshots::load_fixture("regressions/<ID>-<slug>")`,
   replays the input that triggers the bug, and checks the correct
   outcome. Commit both, red, before the fix.

Never delete these files. When the format version changes, regenerate
them (`DRAW_UPDATE_SNAPSHOTS=1`, or re-save from the app) instead.
`core::snapshot::tests::fixtures_are_canonical` checks every file here.
