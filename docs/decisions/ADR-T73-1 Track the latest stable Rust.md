---
id: ADR-T73-1
title: Track the latest stable Rust
status: accepted
kind: decision
date: 2026-11-03
task: T73
builds_on: ["[[ADR-T27-1 Build in an AlmaLinux 8 container for a glibc 2.28 floor]]", "[[ADR-0010 Clippy lint policy]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T73-1 Track the latest stable Rust

## Context

`Cargo.toml` said `rust-version = "1.85"`, but draw no longer builds on
1.85: `fontdue 0.9.4` uses `integer_sign_cast` (stable since 1.87), so
`cargo +1.85 check` fails. Nobody noticed because nothing built on the
declared version. The lint set already follows recent clippy
([[T43 Adopt clippy 1.99 lints]]), and the packaging image pinned 1.97.0.
draw ships as a binary (`.deb`/`.rpm`), not as a library, so an old
minimum Rust helps nobody but people building from source, who use
rustup anyway.

## Decision

- `rust-version` is the latest stable minor release, now **1.99**
  (stable is 1.99.1 on 2026-11-03).
- CI ([[T73 Continuous integration on GitHub]]) runs the gates on the
  current `stable` and a second **msrv** job on exactly `rust-version`,
  so the declared version stays true.
- `packaging/Containerfile` pins a toolchain at least `rust-version`
  (now 1.99.1); `tests/ci.rs` checks this, since cargo refuses to build
  a crate whose `rust-version` is newer than the toolchain.
- The README's build requirement names the same version.
- Raising it again is a one-line change in `Cargo.toml`, the
  Containerfile and the README, done when a task wants a newer feature
  or lint; `tests/ci.rs` fails if the three disagree. No new ADR is
  needed for such a bump.

## Alternatives considered

- **Raise only to 1.87**, the real floor today — keeps source builds on
  older toolchains working, but the owner asked to follow the latest
  Rust, and the floor would drift again with the next dependency update.
- **Drop `rust-version` and the msrv job** — then nothing tells a user
  with an old toolchain why the build fails.
- **`rust-toolchain.toml` pinning a version** — forces a download on
  every machine and in CI and hides what stable does; not needed.

## Consequences

- People building from source need Rust 1.99 or newer.
- When a new stable adds clippy lints, the `gates` job may go red
  without a code change; that is the signal to adopt or allow them, as
  in T43.
- The msrv job and the gates job build almost the same thing right
  after a bump; they diverge as stable moves on.

## Rollback plan

Lower `rust-version` in `Cargo.toml` (and the README line) to the oldest
toolchain that builds, and keep the msrv job pointed at it; record that
in a rollback ADR.
