---
id: ADR-T27-1
title: Build in an AlmaLinux 8 container for a glibc 2.28 floor
status: accepted
kind: decision
date: 2026-10-15
task: T27
builds_on: [ADR-0002, ADR-0014]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T27-1 Build in an AlmaLinux 8 container for a glibc 2.28 floor

## Context

The 1.0.0 binary was built on Fedora 44 and needs `GLIBC_2.43`, so it
does not start on Debian stable, Ubuntu LTS or older Fedora releases
([[T27 Linux packaging]]). A binary only runs where the glibc is at least
as new as the newest symbol version it links. The target floor is glibc
2.28 (RHEL 8, Debian 10, Ubuntu 20.04). The development machine has
`podman` but neither `zig` nor `cargo-zigbuild`.

## Decision

`scripts/package.sh` builds the release binary inside a container made
from `packaging/Containerfile`: AlmaLinux 8 (glibc 2.28), `gcc` and
`binutils` from the distribution, a pinned Rust toolchain from rustup,
and the pinned packaging tools of [[ADR-T27-2 Package formats from one staging tree and one library list]].
The repository is mounted into the container and the build writes to
`target/package/`; the cargo registry lives in a named podman volume so
rebuilds are incremental. After the build, `check_glibc_floor` reads
`objdump -T` and fails if any `GLIBC_` version is above 2.28.

The host needs only `podman`. The image is rebuilt when the Containerfile
changes.

## Alternatives considered

- **`cargo zigbuild --target x86_64-unknown-linux-gnu.2.28`** — no
  container, but needs `zig` plus `cargo-zigbuild` on the host and links
  against zig's glibc stubs rather than a real glibc 2.28; a container
  gives the real thing and also carries the packaging tools.
- **Static musl binary** — loading the system's GL drivers (glibc
  shared objects) from a static musl binary is unreliable.
- **An old Debian/Ubuntu image** — `debian:10` is end of life and its
  archive moved; AlmaLinux 8 is supported until 2029 and has the exact
  floor.

## Consequences

- Packages start on any x86_64 distribution with glibc ≥ 2.28.
- The first `scripts/package.sh` run downloads the image, a toolchain
  and the packaging tools (several minutes); later runs reuse them.
- `scripts/check.sh` is unaffected: everyday builds stay on the host.
- Raising the floor later is one line in the Containerfile plus the
  constant in `check_glibc_floor`.

## Rollback plan

Build on the host again (`cargo build --profile dist`) and drop the
floor check; record it as a rollback ADR, since packages would then
only run on distributions as new as the build machine.
