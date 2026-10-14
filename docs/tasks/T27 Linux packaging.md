---
id: T27
title: Linux packaging
status: ready
wave: 18
branch: task/T27-packaging
depends_on: [T14]
adrs: []
feature: "[[Linux packages]]"
tutorial: "[[27 Shipping a binary to other Linux machines]]"
tags: [task]
---

# T27 Linux packaging

## Goal

`scripts/package.sh` builds release artifacts that install and start on
other Linux machines, not only on the machine that built them: a portable
tarball, a `.deb`, an `.rpm` and a local Flatpak bundle, each with a
desktop entry and icon and with its runtime libraries declared. A missing
library or display gives a short message naming what to install instead
of a panic, and a crash leaves a report that can be traced back to the
source. macOS and Windows stay build-from-source.

Found after the 1.0.0 release (2026-10-14), checking `target/release/draw`:

- `ldd` lists only libc, libm and libgcc. miniquad `dlopen`s
  `libGL.so.1`, `libEGL.so.1`, `libX11.so.6`, `libXi.so.6` and
  `libxkbcommon.so.0` at startup, so package tools that infer dependencies
  from linked libraries miss all of them.
- The binary needs `GLIBC_2.43` (built on Fedora 44), so it does not start
  on Debian stable, Ubuntu LTS or older Fedora releases.
- `panic = "abort"` plus `strip = true`: a crash on a user's machine
  leaves no backtrace and no symbols.
- No `.desktop` file or icon: `cargo install` works, but the app is not in
  application launchers.
- The README lists only `-devel` build headers, not what a prebuilt binary
  needs at runtime.
- On a Wayland session without XWayland the window cannot open
  (miniquad's `X11Only` backend, [[ADR-T12-1 Blocking event loop with cached frame]]).

## Spec

To be refined in the spec step; every AC below must name its tests
there. ADRs expected:

- **ADR-T27-1 — glibc floor and how to build for it.** Target: glibc
  ≤ 2.28 (RHEL 8, Debian 10, Ubuntu 20.04). Options to weigh:
  `cargo zigbuild --target x86_64-unknown-linux-gnu.2.28` or a container
  build on an old distro (podman). Not musl: loading the system's GL
  drivers from a static musl binary is unreliable.
- **ADR-T27-2 — Package formats and tools.** Tarball + `cargo-deb` +
  `cargo-generate-rpm` (build-time tools only; the app keeps its single
  dependency) + Flatpak manifest. Runtime dependencies are declared by
  hand from one list, `packaging/runtime-libs`.
- **ADR-T27-3 — Crash reports.** Keep `panic = "abort"`; a panic hook
  writes the report before the abort. A `dist` profile builds with debug
  info and the package step splits it into `draw.debug`
  (`objcopy --only-keep-debug`), shipped beside the artifacts but not in
  them.

Acceptance criteria:

- **AC-1 — Runtime libraries declared.** `packaging/runtime-libs` maps
  every shared library the binary `dlopen`s to its Fedora/RHEL and
  Debian/Ubuntu package. The `.deb` `Depends` and `.rpm` `Requires` come
  from that list. A check fails if the binary loads a library the list
  does not name (scan the release binary's strings for `lib*.so*`), so a
  macroquad upgrade cannot silently add one.
- **AC-2 — glibc floor.** The packaged binary needs no `GLIBC_` symbol
  version above 2.28 (`objdump -T`), checked by `scripts/package.sh`.
  Owner: the tarball starts on one older distro (e.g. a Debian 12 or
  Ubuntu 22.04 VM or toolbox) on the target.
- **AC-3 — Startup preflight.** Before opening the window,
  `shell::preflight` checks for a display and for the runtime libraries.
  When one is missing, the app prints one message naming what is missing
  and the package to install, then exits with code 1 (no panic, no
  backtrace). Cases: no `DISPLAY` and no `WAYLAND_DISPLAY`; a Wayland
  session with no `DISPLAY` (XWayland missing); a library from AC-1 not
  found. The decision logic is a pure function over the environment and
  the set of libraries found, unit-tested; finding the libraries must not
  need `unsafe` or a new dependency.
- **AC-4 — Crash report.** A panic writes the message, location, app
  version and build id to stderr and to
  `$XDG_STATE_HOME/draw/crash.log` (fallback `~/.local/state/draw/`),
  then aborts as today. Formatting and path choice are pure functions
  with unit tests; the hook itself is checked once by hand.
- **AC-5 — Debug symbols.** `scripts/package.sh` produces `draw.debug`
  matching the shipped binary (same build id), so a crash address from a
  user can be resolved with `addr2line`. The shipped binary stays
  stripped and about 1 MB.
- **AC-6 — Desktop integration.** `packaging/` holds `draw.desktop`, a
  scalable SVG icon plus PNGs (48, 128, 256) and an AppStream
  `metainfo.xml`. `desktop-file-validate` and
  `appstreamcli validate` pass in `scripts/package.sh`; every package
  installs them to the standard paths. `window_conf` sets miniquad's
  `linux_wm_class` to `draw` (today `miniquad-application`) and the
  desktop file's `StartupWMClass` matches, so the running window gets
  the launcher icon. Owner: draw appears in the
  launcher after installing the `.rpm` on the target.
- **AC-7 — Tarball.** `draw-<version>-x86_64-linux.tar.gz` contains the
  binary, desktop file, icons, metainfo, LICENSE, the embedded font's
  license (from T29), README, CHANGELOG and an
  `install.sh` that installs to `~/.local` by default (`--prefix` to
  change) and `--uninstall` that removes exactly what it installed.
- **AC-8 — `.deb` and `.rpm`.** Both build from `scripts/package.sh`,
  install the binary to `/usr/bin/draw` plus the files of AC-6, and
  declare the AC-1 dependencies. Owner: `.rpm` installs, starts and
  uninstalls cleanly on the target; `.deb` the same in a Debian/Ubuntu
  container or VM.
- **AC-9 — Flatpak.** `packaging/flatpak/` has a manifest that builds a
  local bundle with `flatpak-builder` and runs with `--socket=x11`
  (plus `--socket=fallback-x11` / `--share=ipc`) and `--device=dri`, on
  the freedesktop runtime. Owner: the bundle installs and starts on the
  target.
- **AC-10 — README.** Install section split into *Prebuilt packages*
  (Linux: tarball, `.deb`, `.rpm`, Flatpak, and the runtime packages for
  the tarball) and *Build from source* (Linux, macOS, Windows; build
  headers only here). States that macOS and Windows users build from
  source, and that on Wayland draw needs XWayland.
- **AC-11 — Release output.** `scripts/package.sh` writes everything to
  `dist/` (ignored by git) with a `SHA256SUMS` file, and fails if any
  check of AC-1, AC-2, AC-6 fails. `scripts/check.sh` stays green and
  does not need the packaging tools.

## Out of scope

- macOS and Windows packages, signing or notarisation: users there build
  from source.
- Native Wayland backend: [[T28 Experimental native Wayland backend]]
  (it changes the backend choice of
  [[ADR-T12-1 Blocking event loop with cached frame]] and the wake-up of
  [[ADR-T26-1 Reuse the document layer during pan and zoom]]). T27 keeps
  `preflight` and `runtime-libs` easy for T28 to extend.
- Publishing: Flathub submission, distro repositories, CI that builds on a
  tag, GitHub releases.
- Version bump and the release itself.
- AppImage and Snap.

## Files owned

- `packaging/**` (new: runtime-libs, desktop file, icons, metainfo,
  `install.sh`, Flatpak manifest)
- `scripts/package.sh` (new)
- `src/shell/preflight.rs`, `src/shell/crash.rs` (new), `src/shell/mod.rs`,
  `src/main.rs`, `src/shell/app.rs` (`window_conf` WM class only)
- `Cargo.toml` (`dist` profile, `[package.metadata.deb]` and
  `[package.metadata.generate-rpm]`; no new dependencies)
- `tests/packaging.rs` (new: desktop file and runtime-libs checks that need
  no packaging tools)
- `.gitignore` (`/dist`)
- `README.md` (Install section)
- new ADRs `ADR-T27-*`, feature note `docs/features/Linux packages.md`,
  tutorial `docs/tutorials/27 Shipping a binary to other Linux machines.md`

## Subtasks (one commit each)

- [ ] spec — `docs(T27): …`
- [ ] tests — `test(T27): …`
- [ ] models — `feat(T27): …`
- [ ] behaviour — `feat(T27): …`
- [ ] quality — `chore(T27): …`
- [ ] docs — `docs(T27): …`

## Learning path

Step 27 — requires step 12 (app shell) and step 14 (release validation).

## Log

- 2026-10-14 — Created after the 1.0.0 release from a packaging review of
  the release binary (findings in *Goal*). Owner wants every Linux
  problem handled except native Wayland; macOS and Windows build from
  source.
- 2026-10-14 — Moved after T29–T31 (wave 18). The binary then embeds
  T29's font: every package ships its license, and the size budget of
  AC-5 is re-measured with it.
