---
tags: [process]
---

# Commit Convention

Commits are **atomic at the subtask level**: one workflow step of one task =
one commit (see [[Workflow]]). Format is Conventional Commits with the task id
as scope:

```
<type>(<task-id>): <imperative summary, ≤ 72 chars>

<optional body: what and why, wrapped at 72>

Refs: docs/tasks/<task-id> <Title>.md
```

| Step | Type | Example |
|------|------|---------|
| Spec | `docs` | `docs(T04): specify smoothing acceptance criteria` |
| Tests | `test` | `test(T04): add failing tests for RDP simplification` |
| Models | `feat` | `feat(T04): add Smoother and SmoothingLevel types` |
| Behaviour | `feat` | `feat(T04): implement resampling, EMA and RDP` |
| Quality | `refactor` / `chore` | `chore(T04): satisfy clippy and rustfmt` |
| Docs | `docs` | `docs(T04): add feature note and RDP tutorial` |
| Integrate | merge | `Merge branch 'task/T04-smoothing'` + `docs: integrate T04` |
| Rollback | `revert` | `revert(T04): roll back EMA smoothing (ADR-T09-1)` |

## Authorship

- Author and committer: **Gustavo Bender** only.
- **No** `Co-Authored-By`, "Generated with", or any agent/tool mention in
  commit messages or PR descriptions. This applies to every agent working on
  the repo.

## Branches

- `main` — integrated, always green.
- `task/<ID>-<slug>` — one per task, e.g. `task/T04-smoothing`.
- Merge with `--no-ff` so every task stays visible as one unit in history.
