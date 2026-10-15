---
id: T27
title: Linux packaging
status: review
wave: 20
branch: task/T27-packaging
depends_on: [T14]
adrs: [ADR-T27-1, ADR-T27-2, ADR-T27-3]
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

Refined 2026-10-15. Decisions:
[[ADR-T27-1 Build in an AlmaLinux 8 container for a glibc 2.28 floor]],
[[ADR-T27-2 Package formats from one staging tree and one library list]],
[[ADR-T27-3 Crash report from a panic hook and split debug symbols]].

Facts the spec rests on (miniquad 0.4.11, `X11Only` backend, GLX with
EGL fallback): the window needs `libX11.so.6`, `libXi.so.6`,
`libxkbcommon.so.0` and one of `libGL.so.1` / `libEGL.so.1`. The binary
also names `libwayland-client`, `libwayland-egl` and `libdecor-0` (the
unused Wayland backend); they are listed as `unused` so the scan of AC-1
passes and [[T28 Experimental native Wayland backend]] can flip them.

The application id is `io.github.UmBender.draw`: desktop file, icon name,
metainfo and Flatpak all use it (the `draw.desktop` name of the first
draft is replaced, see ADR-T27-2). The window's WM class stays `draw`
and the desktop file's `StartupWMClass=draw` ties the two together.

Acceptance criteria:

- **AC-1 — Runtime libraries declared.** `packaging/runtime-libs` lists
  every shared library the binary may `dlopen` with its role
  (`required`, `gl` = at least one of the group, `unused`) and its
  Fedora/RHEL and Debian/Ubuntu package. `[package.metadata.deb]`
  `depends` and `[package.metadata.generate-rpm.requires]` equal the
  `required` + `gl` packages of the list. The release binary is scanned
  for `lib*.so*` strings in `scripts/package.sh`; the debug binary in a
  test.
  Tests: `preflight::tests::parse_runtime_libs_reads_roles_and_packages`,
  `parse_runtime_libs_rejects_bad_lines`,
  `embedded_runtime_libs_parse`;
  `packaging::library_bases_finds_packed_strings`,
  `packaging::runtime_libs_names_every_library_the_binary_mentions`,
  `packaging::deb_depends_match_runtime_libs`,
  `packaging::rpm_requires_match_runtime_libs`.
- **AC-2 — glibc floor.** The packaged binary needs no `GLIBC_` symbol
  version above 2.28 (`objdump -T`), checked by `scripts/package.sh`
  (`check_glibc_floor`). Owner: the tarball starts on one older distro
  (e.g. a Debian 12 or Ubuntu 22.04 VM or toolbox) on the target.
- **AC-3 — Startup preflight.** Before the window opens, `main` calls
  `shell::preflight::run`. A missing display or library prints one line
  naming what is missing and the package to install and exits with
  code 1. Libraries are found by looking for their soname in
  `/etc/ld.so.cache`, in `LD_LIBRARY_PATH` and in the standard library
  directories (plain file reads: no `unsafe`, no subprocess, no new
  dependency). `DRAW_SKIP_PREFLIGHT=1` skips the check. Only Linux runs
  it.
  Tests: `preflight::tests::check_no_display_reports_no_display`,
  `check_wayland_without_display_reports_xwayland`,
  `check_x11_display_with_all_libs_is_ok`,
  `check_missing_required_lib_names_its_packages`,
  `check_one_gl_lib_is_enough`,
  `check_no_gl_lib_names_both`,
  `check_unused_libs_are_not_needed`,
  `check_skip_variable_skips_everything`,
  `cache_contains_soname_matches_whole_names_only`,
  `problem_message_is_one_line`.
- **AC-4 — Crash report.** A panic writes the message, location, app
  version, build id and Unix time to stderr and appends it to
  `$XDG_STATE_HOME/draw/crash.log` (fallback `~/.local/state/draw/`),
  then aborts as today. Formatting, path choice and reading the GNU
  build id from the ELF file are pure functions.
  Tests: `crash::tests::crash_log_path_prefers_absolute_xdg_state_home`,
  `crash_log_path_ignores_relative_or_empty_xdg_state_home`,
  `crash_log_path_without_home_is_none`,
  `format_report_has_message_location_version_and_build_id`,
  `panic_message_reads_str_and_string_payloads`,
  `format_report_marks_unknown_fields`,
  `build_id_reads_gnu_note`, `build_id_of_elf_without_note_is_none`,
  `build_id_of_garbage_is_none` (synthetic ELF files: the host linker
  does not always add a build id, so `scripts/package.sh` passes
  `-Wl,--build-id` and checks it, AC-5).
  The hook itself is checked once by hand (log of the check in *Log*).
- **AC-5 — Debug symbols.** `scripts/package.sh` builds with the `dist`
  profile (release + debug info, not stripped), writes `draw.debug`
  with `objcopy --only-keep-debug`, strips the shipped binary and adds a
  `.gnu_debuglink`; it checks both have the same build id and that the
  shipped binary stays under 2 MB (`check_debug_symbols`).
- **AC-6 — Desktop integration.** `packaging/` holds
  `io.github.UmBender.draw.desktop`, the icons of `packaging/icons/`
  (SVG + 48, 128, 256 PNG) and `io.github.UmBender.draw.metainfo.xml`.
  `desktop-file-validate` and `appstreamcli validate` pass in
  `scripts/package.sh`; every package installs them under
  `share/applications`, `share/icons/hicolor/<size>/apps` and
  `share/metainfo`. `window_conf` sets `linux_wm_class` to `draw`.
  Tests: `app::tests::window_conf_sets_wm_class`;
  `packaging::desktop_file_matches_wm_class_and_icon`,
  `packaging::metainfo_release_is_crate_version`,
  `packaging::icons_have_every_size`. Owner: draw appears in the
  launcher with its icon after installing the `.rpm` on the target.
- **AC-7 — Tarball.** `draw-<version>-x86_64-linux.tar.gz` holds the
  binary, desktop file, icons, metainfo, LICENSE, the font licenses,
  README, CHANGELOG and `install.sh` (default prefix `~/.local`,
  `--prefix DIR`, `--uninstall` removes exactly the files it installs).
  `scripts/package.sh` installs and uninstalls it into a temporary
  prefix and fails if a file is missing or left behind
  (`check_tarball_install`).
- **AC-8 — `.deb` and `.rpm`.** Built by `cargo-deb` and
  `cargo-generate-rpm` inside the build container from the same staging
  tree; `/usr/bin/draw` plus the AC-6 files and the licenses, with the
  AC-1 dependencies (covered by the AC-1 tests). Owner: `.rpm` installs,
  starts and uninstalls on the target; `.deb` the same in a
  Debian/Ubuntu container or VM.
- **AC-9 — Flatpak.** `packaging/flatpak/io.github.UmBender.draw.yml`
  builds a local bundle with `flatpak-builder` from the staged files on
  `org.freedesktop.Platform` 26.08, with `--socket=x11`,
  `--share=ipc` and `--device=dri`. `scripts/package.sh` builds it when
  `flatpak-builder` (or `org.flatpak.Builder`) is available and says so
  when it is not. Owner: the bundle installs and starts on the target.
- **AC-10 — README.** Install section split into *Prebuilt packages*
  and *Build from source*, runtime packages for the tarball, XWayland
  note, crash log location and `DRAW_SKIP_PREFLIGHT`.
- **AC-11 — Release output.** `scripts/package.sh` needs only `podman`
  (plus `flatpak-builder` for AC-9), writes everything to `dist/`
  (ignored by git) with `SHA256SUMS`, and fails if any check of AC-1,
  AC-2, AC-5, AC-6 or AC-7 fails. `scripts/check.sh` stays green and
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
  `install.sh`, `Containerfile`, `build.sh` run in the container,
  Flatpak manifest)
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

- [x] spec — `docs(T27): …`
- [x] tests — `test(T27): …`
- [x] models — `feat(T27): …`
- [x] behaviour — `feat(T27): …`
- [x] quality — `chore(T27): …`
- [x] docs — `docs(T27): …`

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
- 2026-10-15 — Owner supplied the app icon (sword and crown):
  `packaging/icons/draw.svg` plus `draw-48.png`, `draw-128.png`,
  `draw-256.png`. AC-6 uses these files; install them as
  `hicolor/{scalable,48x48,128x128,256x256}/apps/draw.{svg,png}`.
- 2026-10-15 — Implemented. `scripts/package.sh --no-flatpak` passes in
  the AlmaLinux 8 container: newest symbol `GLIBC_2.28`, build id shared
  by `draw` and `draw.debug`, stripped binary 1 442 688 bytes, desktop
  entry and metainfo valid, tarball installs 12 files and removes them
  all. `dist/` holds the tarball (727 kB), `.deb` (596 kB), `.rpm`
  (647 kB), `draw.debug` and `SHA256SUMS`.
- 2026-10-15 — Deviations: EL8 ships no `appstreamcli`, so inside the
  container the metainfo is checked with `appstream-util validate-relax`
  (`libappstream-glib`); `appstreamcli validate` is used where present
  (host: 1.1.3, passes). Icons are installed as
  `hicolor/*/apps/io.github.UmBender.draw.{svg,png}`, not `draw.*`, so the
  desktop entry's `Icon=` is the application id (AC-6, ADR-T27-2).
- 2026-10-15 — Hand checks (AC-3, AC-4) on the packaged binary: without
  `DISPLAY` / `WAYLAND_DISPLAY` it prints the no-display line and exits 1;
  with `WAYLAND_DISPLAY` only, the Xwayland line; in a bare
  `almalinux:8` container with `DISPLAY=:0` it names the four missing
  sonames and their packages and exits 1. With `DRAW_SKIP_PREFLIGHT=1`
  and no display, miniquad's `XOpenDisplay() failed!` panic is written
  to stderr and appended to `$XDG_STATE_HOME/draw/crash.log` with the
  binary's build id, then the process aborts (exit 134).
- 2026-10-15 — Still for the owner: launcher and icon after installing
  the `.rpm` (AC-6, AC-8), the `.deb` in a Debian/Ubuntu container or VM
  (AC-8), the tarball on an older distro (AC-2), and the Flatpak bundle
  (AC-9: `flatpak-builder` is not installed here, so it was not built).
