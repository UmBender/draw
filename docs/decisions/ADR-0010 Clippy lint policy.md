---
id: ADR-0010
title: Clippy lint policy
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0008]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0010 Clippy lint policy

## Context

Clippy is the code-quality gate. Pedantic lints catch real issues but some are
noise in graphics code (float/int casts are routine).

## Decision

Lint levels live in `Cargo.toml` `[lints]` so every tool and agent uses the same
set:

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
cast_precision_loss = "allow"
cast_possible_truncation = "allow"
cast_sign_loss = "allow"
module_name_repetitions = "allow"
float_cmp = "deny"
unwrap_used = "deny"
expect_used = "warn"
dbg_macro = "deny"
todo = "warn"
```

The gate runs `cargo clippy --all-targets -- -D warnings`, so warnings fail the
build too. `#[allow(...)]` needs a comment explaining why. Tests may use
`unwrap`/`expect` (via `#[cfg_attr(test, allow(...))]` in the test module).

## Alternatives considered

- **Default clippy only** — misses useful pedantic checks.
- **Full pedantic + nursery as errors** — too noisy for f32-heavy code.

## Consequences

- `todo!()` placeholders from the *models* step warn, so the *quality* step
  must remove them all.

## Rollback plan

Adjust levels with an amending ADR.
