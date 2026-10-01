# draw

Minimal, fast, keyboard-first scratch canvas for competitive programming (Rust + macroquad).

## Where things are

- `docs/` — Obsidian vault: tasks, decisions (ADRs), architecture, features, tutorials.
  Start at `docs/Home.md`.
- `src/core/` — pure logic, no macroquad, fully tested and fuzzed.
- `src/shell/` — macroquad window, input translation, rendering, toolbar.
- `scripts/check.sh` — all quality gates (fmt, clippy -D warnings, tests, docs).

## How to work

- Implement tasks with the `implement-task` skill (`/implement-task T04`).
  Process: `docs/process/Workflow.md`. Rust rules: `rust-skills` skill.
- Tests are written and committed **before** the code (SDD + TDD).
- One branch per task, one commit per workflow step.
- Architecture decisions are append-only ADRs (`docs/process/Decision Protocol.md`).
  Rollbacks are new ADRs plus `git revert` — never rewrite history.
- **Commits are authored by the user only: never add `Co-Authored-By` or mention any
  agent/AI in commits, PRs or docs.**
- Parallel work: `docs/process/Parallel Execution.md`. Only the integrator edits
  `Task Board`, `Decision Log`, `Feature Index`, `Learning Path`.
