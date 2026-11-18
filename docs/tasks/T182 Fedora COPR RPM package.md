---
id: T182
title: Fedora COPR RPM package
status: todo
wave: 75
branch: task/T182-fedora-copr-rpm
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, packaging]
---

# T182 Fedora COPR RPM package

## Goal

`draw` installs on Fedora via a self-managed COPR repository (`sudo dnf copr enable <user>/draw && sudo dnf install draw`) without waiting for Fedora official package review.

## Known facts (2026-11-18)

- `scripts/package.sh` already builds an `.rpm` locally, while COPR builds `.srpm` -> `.rpm` in clean mock chroots without network access during `%build`.
- `packaging/runtime-libs` lists the exact `dlopen` sonames needed at runtime (`libX11`, `libXi`, `libxkbcommon`, `libGL`, `libEGL`, `libwayland-client`, `libwayland-egl`, optional `libdecor-0`).
- Fedora COPR requires only a Fedora Account (FAS); no external package review is required.

## Spec

- **AC-1** — `packaging/rpm/draw.spec` builds `draw` offline (`cargo build --release --locked --offline` using a vendored source tarball or `Source1` vendor archive) and installs `/usr/bin/draw`, desktop entry, metainfo, and icons.
- **AC-2** — `Requires:` in `draw.spec` covers every `required`, `gl`, and `wayland` library from `packaging/runtime-libs` (`libX11`, `libXi`, `libxkbcommon`, `libglvnd-glx`, `libglvnd-egl`, `wayland-client`, `wayland-egl`).
- **AC-3** — `tests/packaging.rs` verifies `draw.spec` `Requires:` matches `packaging/runtime-libs` and `Version:` matches `Cargo.toml`.
- **AC-4** — `scripts/copr-check.sh` builds the SRPM and verifies `rpmbuild` in a Fedora container (plus `draw --version` and preflight check).
- **AC-5** — `README.md`: an "Install on Fedora (COPR)" section (`dnf copr enable` + `dnf install draw`).

## Out of scope

- Submitting `draw` to the official Fedora repositories (Bugzilla package review + `rust2rpm` unbundling).

## Files owned

- `packaging/rpm/draw.spec`
- `scripts/copr-check.sh`, `scripts/package.sh` (if sharing vendor tarball generation)
- `tests/packaging.rs`, `README.md` (Fedora COPR install section)
- this note

## Subtasks

- [ ] tests — `test(T182): check Fedora spec dependencies against runtime-libs`
- [ ] behaviour — `feat(T182): add Fedora COPR spec and rpmbuild check`
- [ ] docs — `docs(T182): document Fedora COPR installation in README`
- [ ] publish — configure COPR project and trigger initial build
