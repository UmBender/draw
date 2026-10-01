---
tags: [process]
---

# Quality Gates

All gates run with one command:

```sh
scripts/check.sh          # fmt check, clippy -D warnings, tests, docs
scripts/check.sh --fuzz   # same + long fuzz run (PROPTEST_CASES=20000)
```

| Gate | Command | Must |
|------|---------|------|
| Format | `cargo fmt --all -- --check` | no diff |
| Lint | `cargo clippy --all-targets -- -D warnings` | zero warnings (lint levels in `Cargo.toml` `[lints]`, see [[ADR-0010 Clippy lint policy]]) |
| Tests | `cargo test --all-targets` | all pass, including property tests |
| Docs | `cargo doc --no-deps` with `RUSTDOCFLAGS=-D warnings` | no broken intra-doc links |
| Fuzz | `PROPTEST_CASES=20000 cargo test --test fuzz` | no failure (see [[ADR-0009 Fuzzing with proptest]]) |

## Definition of Done (per task)

- [ ] Every `AC-n` in the spec has at least one passing test.
- [ ] `scripts/check.sh` is green on the branch head.
- [ ] Only files in *Files owned* were touched.
- [ ] Feature note + tutorial note written; learning-path position stated.
- [ ] New architecture choices recorded as ADRs.
- [ ] Commit history follows [[Commit Convention]] (one commit per step).
- [ ] Task note status is `review`.

## Code rules

Follow the `rust-skills` skill. Most relevant categories for this project:
`own-`, `err-`, `mem-`, `api-`, `num-` (floats/NaN!), `type-`, `pat-`,
`name-`, `test-`, `doc-`, `perf-`, `proj-`, `lint-`, `anti-`. Not relevant:
`async-`, `conc-`, `serde-`, `unsafe-` (no `unsafe` allowed at all).
