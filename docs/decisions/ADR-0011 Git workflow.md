---
id: ADR-0011
title: Git workflow
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0001, ADR-0008]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0011 Git workflow

## Context

Tasks are implemented in branches, possibly by parallel agents; history must
show each task as a unit with atomic subtask commits, attributed only to the
user.

## Decision

- One branch per task: `task/<ID>-<slug>`, created from `main`.
- One commit per workflow step (spec, tests, models, behaviour, quality,
  docs) — see [[Commit Convention]].
- Integration: `git merge --no-ff` in dependency order, followed by a
  `docs: integrate <ID>` commit that updates the shared indexes.
- Parallel agents use isolated git worktrees (see [[Parallel Execution]]).
- Author: Gustavo Bender only. No `Co-Authored-By` or agent mentions.
- No history rewriting on `main`: no force-push, no squash merges. Rollbacks
  are `git revert` + a rollback ADR (see [[Decision Protocol]]).
- `docs/` (the Obsidian vault) lives in the same repository so decisions and
  code move together.

## Alternatives considered

- **Squash merges** — lose the atomic TDD steps the user asked for.
- **Separate docs repository** — decisions drift from the code they describe.

## Consequences

- Task branches contain red commits by design; bisect on `main` merge commits
  (`git bisect --first-parent`).

## Rollback plan

Process change → amending ADR.
