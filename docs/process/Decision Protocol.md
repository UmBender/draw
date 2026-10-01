---
tags: [process]
---

# Decision Protocol — transactional, append-only

Architecture decisions are recorded as ADRs in `decisions/`. The set of ADRs
is a **log**, like a database transaction log: entries are only ever *added*.

## Rules

1. **Never rewrite a decision.** Once an ADR is `accepted`, its *Context*,
   *Decision* and *Consequences* sections are frozen. Only its `status` and
   `superseded_by` / `reverted_by` frontmatter may change.
2. **Every ADR states its place in the chain** via frontmatter:
   - `builds_on`: ADRs this one assumes.
   - `amends`: ADRs it partially changes (old one stays `accepted`).
   - `supersedes`: ADRs it fully replaces (old one becomes `superseded`).
   - `reverts`: ADRs it rolls back (old one becomes `reverted`).
3. **Rollbacks are explicit.** Undoing a decision is a *new* ADR with
   `kind: rollback` and `reverts: [...]`, explaining why and what the code
   returns to. The code change is a `revert(...)` commit referencing it
   (see [[Commit Convention]]). History is never rewritten (no force-push,
   no squashing away the original).
4. **Ids**: foundation ADRs are `ADR-NNNN`. ADRs born inside a task are
   `ADR-<task>-<n>` (see [[Parallel Execution]]).
5. **Order** of the chain is the order of rows in [[Decision Log]], appended at
   integration time.

## Statuses

`proposed` → `accepted` → (`superseded` | `reverted`)

`provisional` — accepted, but explicitly waiting for input that may replace it
(e.g. [[ADR-0012 Fixed palette and theme tokens]] was provisional until [[ADR-0015 Kanagawa Dragon theme]] supplied the theme).

## Template

Use `templates/ADR.md`.
