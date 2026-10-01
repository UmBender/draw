---
tags: [process]
---

# Parallel Execution

Tasks are cut so that independent ones can be implemented **at the same time
by separate agents**, each in its own git worktree, without merge conflicts.

## How conflicts are avoided

1. **Module skeleton up front** — [[T00 Bootstrap]] creates every module file
   and every `mod` declaration. Later tasks only *fill* files they own; nobody
   edits `lib.rs` or `Cargo.toml` except the task that owns them.
2. **Files owned** — each task note lists the files it may touch. Two tasks in
   the same wave never own the same file.
3. **Namespaced ADR ids** — ADRs created inside a task are named
   `ADR-<task>-<n>` (e.g. `ADR-T04-1`), so parallel branches never pick the
   same number. Order in the chain is fixed at integration time in
   [[Decision Log]].
4. **Shared indexes are integrator-only** — [[Task Board]], [[Decision Log]],
   [[Feature Index]] and [[Learning Path]] are updated only on `main` during
   integration, which is serialized.

## Orchestration

```
integrator (main session)
 ├─ find tasks with status ready (all depends_on are done)
 ├─ for each: spawn an agent in an isolated worktree
 │     prompt: "Use the implement-task skill for <ID>"
 ├─ wait for agents → each leaves its branch in status review
 ├─ verify: scripts/check.sh on each branch, review diff vs Files owned
 ├─ merge in dependency order (--no-ff), run check.sh on main, update indexes
 └─ clear context (/clear) — resume next wave from Task Board, not from memory
```

Agents must be told: commits are authored by Gustavo Bender only, no
`Co-Authored-By` or agent mention (see [[Commit Convention]]).

## Waves

See the dependency graph in [[Task Board]]. A wave is the set of tasks whose
dependencies are all in earlier waves; every task in a wave can run in parallel.
