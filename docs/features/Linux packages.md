---
title: Linux packages
task: "[[T27 Linux packaging]]"
adrs: ["[[ADR-T27-1 Build in an AlmaLinux 8 container for a glibc 2.28 floor]]", "[[ADR-T27-2 Package formats from one staging tree and one library list]]", "[[ADR-T27-3 Crash report from a panic hook and split debug symbols]]"]
tutorial: "[[27 Shipping a binary to other Linux machines]]"
shortcuts: []
tags: [feature]
---

# Linux packages

## What it does

draw ships as an `.rpm`, a `.deb`, a local Flatpak bundle and a portable
tarball that start on any x86_64 distribution with glibc 2.28 or newer and
appear in the application launcher with their icon. A missing display or
library gives one line naming the package to install; a crash leaves a
report that points at the source line.

## How to use

| Action | Command |
|--------|---------|
| Build every package into `dist/` | `scripts/package.sh` (needs `podman`) |
| Skip the Flatpak bundle | `scripts/package.sh --no-flatpak` |
| Install the tarball into `~/.local` | `draw-*/install.sh` (`--prefix DIR`) |
| Remove it again | `draw-*/install.sh --uninstall` |
| Start without the startup check | `DRAW_SKIP_PREFLIGHT=1 draw` |
| Read the last crash | `~/.local/state/draw/crash.log` |
| Resolve a crash address | `addr2line -e draw.debug <offset>` |

## How it works

- `scripts/package.sh` builds an AlmaLinux 8 image from
  `packaging/Containerfile` and runs `packaging/build.sh` in it: a `dist`
  profile build with debug info, `draw.debug` split off with `objcopy`,
  the stripped binary plus desktop entry, icons, metainfo and licenses
  staged under `target/package/stage/usr`, then the tarball, `cargo deb`
  and `cargo generate-rpm` from that tree. The checks (library list,
  glibc floor, build ids and size, desktop file validation — with
  `appstream-util`, as EL8 has no `appstreamcli` — tarball
  install/uninstall) run in the same container; the Flatpak manifest
  `packaging/flatpak/io.github.UmBender.draw.yml` copies the staged tree
  on the host.
- `packaging/runtime-libs` lists every library miniquad may `dlopen`; the
  package dependencies in `Cargo.toml` and the startup check both follow
  it, and `tests/packaging.rs` fails when the binary names one it lacks.
- `src/shell/preflight.rs` checks `DISPLAY` / `WAYLAND_DISPLAY` and looks
  the sonames up in `/etc/ld.so.cache`, `LD_LIBRARY_PATH` and the
  library directories before the window opens.
- `src/shell/crash.rs` installs a panic hook that writes message,
  location, version, GNU build id and time to stderr and the crash log
  before the abort.

## Limits

- x86_64 Linux only; macOS and Windows build from source.
- X11 only: Wayland sessions need XWayland
  ([[T28 Experimental native Wayland backend]]).
- Nothing is published or signed; the Flatpak bundle is local, not on
  Flathub.
- The startup check knows the usual library paths; on NixOS or with
  libraries in unusual places use `DRAW_SKIP_PREFLIGHT=1`.
