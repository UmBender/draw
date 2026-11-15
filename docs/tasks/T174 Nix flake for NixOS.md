---
id: T174
title: Nix flake for NixOS
status: in-progress
wave: 72
branch: task/T174-nix-flake-for-nixos
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, packaging]
---

# T174 Nix flake for NixOS

## Goal

draw installs on NixOS from the repository: `nix run github:UmBender/draw`, `nix profile install`, or as a package in a NixOS / home-manager configuration.

## Known facts (2026-11-15)

- The owner is moving to NixOS (2026-11-15). The tarball, .deb and .rpm (ADR-T27-1, ADR-T27-2) do not run there: the binary expects `/lib64/ld-linux-x86-64.so.2`, and the libraries are found through `ld.so.cache`.
- draw loads `libX11`, `libXi`, `libxkbcommon`, `libGL`, `libEGL`, `libwayland-client`, `libwayland-egl` and optionally `libdecor-0` with `dlopen` (`packaging/runtime-libs`). The preflight (`src/shell/preflight.rs`) looks them up in `ld.so.cache` **or** `LD_LIBRARY_PATH`; on NixOS only the second can work, so the package wraps the binary with `LD_LIBRARY_PATH` set to those libraries (dlopen honours it too). The GL drivers come from `/run/opengl-driver` through libglvnd, as for any NixOS GL program.
- `rust-version = "1.99"`: nixpkgs may lag, so the flake pins a toolchain through `rust-overlay` (or `fenix`), with the version written once.
- `vendor/miniquad` is a path dependency and builds offline; `cargoLock.lockFile = ./Cargo.lock`.
- The flatpak manifest (`packaging/flatpak/`) stays; this does not replace it.
- **Owner decision (2026-11-15):** a flake in the repo, tested by building it in a `nixos/nix` container with podman (it pulls the image and the binary cache).

## Spec

Each criterion names its check. Design: `flake.nix` holds the library list
as `{ soname = "libX11.so.6"; pkg = pkgs.libx11; }` entries (one `soname =`
string per library, plain-text parsable); `scripts/nix-check.sh` runs the
container build. Inputs: `nixos-unstable`, `rust-overlay`, `flake-utils`
is avoided (plain `genAttrs`). The toolchain version is written once in
`flake.nix` (`rustVersion = "1.99.0"`).

Tests: AC-1 `flake_exposes_package_app_shell_and_overlay`; AC-3
`flake_libraries_match_runtime_libs`, `flake_toolchain_matches_rust_version`;
AC-2 and AC-4 `readme_has_nixos_install_section`; AC-2 run by
`scripts/nix-check.sh` (not a cargo test: needs podman and network).

- **AC-1** — `flake.nix` (with `flake.lock`) exposes `packages.x86_64-linux.default` (also `aarch64-linux` if it evaluates), `apps.default`, `devShells.default` (the toolchain, sway/Xvfb tools for e2e where nixpkgs has them) and `overlays.default`. The package installs the binary, the `.desktop` file, the icons and the metainfo like `packaging/install.sh`, and wraps the binary with `LD_LIBRARY_PATH`.
- **AC-2** — `nix build` and `nix flake check` pass in a `nixos/nix` container (`podman run`); `result/bin/draw --version` prints the version and the preflight passes with the wrapped path (a check inside the build or the container: `draw --help` and the preflight's library lookup, without a display).
- **AC-3** — A test in `tests/packaging.rs` fails when the library list in the flake drifts from the `required`, `gl`, `wayland` and `optional` lines of `packaging/runtime-libs`.
- **AC-4** — README: an "Install on NixOS" section (`nix run`, `nix profile install`, a NixOS `environment.systemPackages` / home-manager snippet with the flake input).
- **AC-5** — Hand check (owner, on NixOS): draw starts on Wayland and X11 and the tablet works.

## Out of scope

- Submitting draw to nixpkgs; a NixOS module with options; CI with Nix.

## Files owned

- `flake.nix`, `flake.lock`, `nix/**`
- `tests/packaging.rs`, `README.md` (the install section), `docs/**` mentions
- the ADR this task writes (`ADR-T174-*`)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T174): specify the flake`
- [ ] tests — `test(T174): add the flake library list check`
- [ ] behaviour — `feat(T174): add the Nix flake`
- [ ] docs — `docs(T174): document the NixOS install`
- [ ] by hand — owner, on NixOS, recorded in the Log

## Log

- 2026-11-15 — written for the owner's move to NixOS.
