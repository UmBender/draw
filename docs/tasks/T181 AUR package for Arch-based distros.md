---
id: T181
title: AUR package for Arch-based distros
status: done
wave: 75
branch: task/T181-aur-package
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, packaging]
---

# T181 AUR package for Arch-based distros

## Goal

`draw` installs on Arch Linux and Arch-based distros from the AUR (`yay -S draw` / `paru -S draw`, or `makepkg -si`) without needing external package-review approval.

## Known facts (2026-11-18)

- `draw` loads `libX11`, `libXi`, `libxkbcommon`, `libGL`, `libEGL`, `libwayland-client`, `libwayland-egl` and optionally `libdecor-0` with `dlopen` (`packaging/runtime-libs`).
- `packaging/install.sh` already defines the desktop entry (`packaging/io.github.UmBender.draw.desktop`), AppStream metainfo (`packaging/io.github.UmBender.draw.metainfo.xml`), and hicolor icons (`packaging/icons/`).
- Publishing to the AUR (`ssh://aur@aur.archlinux.org/draw.git`) requires only an AUR account and SSH key; there is no external review queue.

## Spec

- **AC-1** — `packaging/aur/PKGBUILD` (and generated `packaging/aur/.SRCINFO`) builds `draw` from the release tarball/tag with `cargo build --release --locked` and installs `/usr/bin/draw`, desktop entry, metainfo, and icons.
- **AC-2** — `depends` in `PKGBUILD` maps every `required`, `gl`, and `wayland` soname in `packaging/runtime-libs` to its Arch package (`libx11`, `libxi`, `libxkbcommon`, `libglvnd`, `wayland`); `makedepends` includes `rust`.
- **AC-3** — A test in `tests/packaging.rs` checks that `packaging/aur/PKGBUILD` `depends` stays in sync with `packaging/runtime-libs` and its `pkgver` matches `Cargo.toml`.
- **AC-4** — `scripts/aur-check.sh` validates the `PKGBUILD` in an Arch container (`makepkg` or `namcap`, and `draw --version` / preflight library lookup).
- **AC-5** — `README.md`: an "Install on Arch Linux (AUR)" section (`yay -S draw` / `paru -S draw`).

## Out of scope

- Promoting `draw` from the AUR to the official Arch `extra` repository.

## Files owned

- `packaging/aur/PKGBUILD`, `packaging/aur/.SRCINFO`
- `scripts/aur-check.sh`
- `tests/packaging.rs`, `README.md` (Arch install section)
- this note

## Subtasks

- [x] tests — `test(T181): check AUR PKGBUILD dependencies against runtime-libs`
- [x] behaviour — `feat(T181): add AUR PKGBUILD and container check script`
- [x] docs — `docs(T181): document AUR installation in README`
- [ ] publish — push `PKGBUILD` and `.SRCINFO` to `aur.archlinux.org/draw.git`
