---
name: implement-task
description: Implement one task from docs/tasks (e.g. "T04") in the draw repo following the SDD+TDD workflow — task branch, spec → failing tests → models → behaviour → clippy/fmt quality → docs, one atomic commit per step, Rust code following the rust-skills rules. Use whenever asked to implement, continue or finish a task by id.
argument-hint: <task-id, e.g. T04>
---

# implement-task

Implements task `$ARGUMENTS` end to end. The source of truth for the process is
the Obsidian vault in `docs/`. Read these before starting:

- `docs/process/Workflow.md` — the steps
- `docs/process/Commit Convention.md` — commit format and **authorship rule**
- `docs/process/Quality Gates.md` — gates and Definition of Done
- `docs/process/Decision Protocol.md` — how to record decisions
- `docs/tasks/$ARGUMENTS *.md` — the task note (spec, files owned, subtasks)
- `docs/architecture/Architecture.md` — module boundaries

Coding rules come from the vendored **rust-skills** skill
(`.claude/skills/rust-skills/SKILL.md`, rule files in `rules/`). Read the
categories listed in *Quality Gates → Code rules* that apply to the task and
open individual rule files when a rule is relevant.

## Hard rules

1. **Authorship**: commits are by the configured git user only. Never add
   `Co-Authored-By`, "Generated with", or any mention of an AI/agent to a commit
   message, branch name, PR, or doc.
2. **Tests before code**: the `test` commit must come before any commit that
   adds the code under test.
3. **Files owned**: touch only files listed in the task note (plus the task's
   own docs: its task note, new feature/tutorial notes, new `ADR-<task>-<n>`
   notes). If you need another file, stop and report — do not edit it.
4. **Do not edit shared indexes** (`Task Board.md`, `Decision Log.md`,
   `Feature Index.md`, `Learning Path.md`) — the integrator does that on `main`.
5. **Never rewrite an accepted ADR.** New decisions → new ADR file from
   `docs/templates/ADR.md` with id `ADR-$ARGUMENTS-<n>` and correct
   `builds_on`/`amends`/`supersedes`/`reverts`.
6. No `unsafe`, no new dependencies (would need an ADR and the owner of
   `Cargo.toml`).

## Procedure

0. **Preflight** — confirm every `depends_on` task has `status: done` (or its
   branch is merged into your base). Working tree must be clean.
1. **Branch** — `git switch -c <branch from task note> main` (skip if already in
   a dedicated worktree on that branch).
2. **Spec** — refine *Spec*: each `AC-n` names concrete test function(s). Add
   ADRs for new architecture choices. Set `status: in-progress`.
   Commit `docs(<ID>): specify <topic> acceptance criteria`.
3. **Tests** — write tests for every `AC-n` (unit tests in a
   `#[cfg(test)] mod tests` with `use super::*;`, proptest for properties,
   `tests/` for integration). Names: `<subject>_<condition>_<expected>`,
   Arrange–Act–Assert. Compare floats with `geom::approx_eq`, never `==`.
   Run `cargo test` and confirm they fail (or don't compile) for the right
   reason. Commit `test(<ID>): add failing tests for <topic>`.
4. **Models** — types, structs, enums, signatures, docs (`missing_docs` is
   on). Bodies may be `todo!()`. `cargo build` must pass.
   Commit `feat(<ID>): add <types>`.
5. **Behaviour** — implement until `cargo test` is green. Handle non-finite
   floats explicitly. No panics on user input.
   Commit `feat(<ID>): implement <behaviour>`.
6. **Quality** — run `scripts/check.sh` until green (fmt, clippy
   `-D warnings`, tests, docs). Remove every `todo!()` you added, refactor
   without changing behaviour. Commit `chore(<ID>): pass clippy and rustfmt`
   (or `refactor(<ID>): …` if code was restructured). If nothing changed, skip
   the commit and note it in the task log.
7. **Docs** — from `docs/templates/`:
   - feature note in `docs/features/` (if the task has a `feature`),
   - tutorial note in `docs/tutorials/` named as in the task frontmatter,
     with `step` and `requires` set for the [[Learning Path]],
   - update `docs/architecture/Architecture.md` only if the task changed it,
   - task note: tick subtasks, fill *Log*, set `status: review`.
   Commit `docs(<ID>): add feature note and tutorial`.
8. **Report** — print: branch name, commit list (`git log --oneline main..`),
   check.sh result, any deviations from the spec or files owned, any new ADRs.

## Commit message template

```
<type>(<ID>): <imperative summary ≤ 72 chars>

<what and why, wrapped at 72>

Refs: docs/tasks/<ID> <Title>.md
```

Use `git commit -F <file>` or a heredoc; never pass `--author` or trailers.
