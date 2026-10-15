---
title: Shipping a binary to other Linux machines
step: 27
requires: ["[[12 The app loop, idle redraw and UI]]", "[[14 Profiling and shipping a release build]]"]
feature: "[[Linux packages]]"
code: ["scripts/package.sh", "packaging/build.sh", "packaging/Containerfile", "packaging/runtime-libs", "packaging/install.sh", "packaging/flatpak/io.github.UmBender.draw.yml", "src/shell/preflight.rs", "src/shell/crash.rs", "src/main.rs", "tests/packaging.rs", "Cargo.toml"]
tags: [tutorial]
---

# Shipping a binary to other Linux machines

> **Learning path step 27.** Requires: step 12 (app shell), step 14
> (release build).

## Why this matters here

`cargo build --release` gives a binary that runs on the machine that
built it. The 1.0.0 binary, built on Fedora 44, failed on Debian stable
before printing a byte, would have panicked on a machine without
`libXi`, and on a crash said only "Aborted". Packaging is the work of
making the binary honest about what it needs from the system it lands
on.

## The concept

Three things decide whether a Linux binary starts elsewhere:

1. **The glibc symbol versions it links.** Every glibc function carries a
   version (`memcpy@GLIBC_2.14`); the dynamic linker refuses to start a
   binary that asks for a version newer than the installed glibc. The
   versions come from the glibc you *link against*, so the fix is to
   build on an old one:

   ```text
   built on glibc 2.43  ──▶ needs GLIBC_2.43 ──▶ fails on Debian 12 (2.36)
   built on glibc 2.28  ──▶ needs ≤ GLIBC_2.28 ──▶ runs on 2.28 … 2.43
   ```

2. **The libraries it loads.** `ldd` shows `NEEDED` entries; libraries
   opened with `dlopen` at run time are invisible to it and to every
   package tool that infers dependencies from it. miniquad opens libX11,
   libXi, libxkbcommon and libGL/libEGL this way.
3. **What it says when something is missing.** A `dlopen` failure deep in
   a dependency is a panic; checking first turns it into an instruction.

And one decides whether a crash can be traced: the shipped binary is
stripped, so the symbols live in a separate file paired with it by the
**GNU build id**, a hash the linker writes into an ELF note.

## How draw implements it

- **Old glibc in a container.** `scripts/package.sh` builds the image of
  `packaging/Containerfile` (AlmaLinux 8, glibc 2.28, pinned Rust) and
  runs `packaging/build.sh` inside it with the repository mounted.
  `check_glibc_floor` reads `objdump -T` and fails above 2.28.
- **One library list.** `packaging/runtime-libs` gives each soname a role
  and its Fedora and Debian package. `tests/packaging.rs` scans the
  binary for `lib*.so` strings (`library_bases`) and fails on any it does
  not list, and compares the list with `[package.metadata.deb] depends`
  and `[package.metadata.generate-rpm.requires]`.
- **Preflight.** `main` calls `shell::preflight::run` before the window.
  `check` is pure: it takes an `Environment` and a `found` closure, so the
  tests feed it every combination of display variables and missing
  libraries. `run` reads the real environment and looks sonames up with
  `cache_contains_soname` over `/etc/ld.so.cache` — just bytes, a soname
  must sit between a NUL or `/` and a NUL — then in the library
  directories. A `gl` group passes when any member is found.
- **Crash report.** `shell::crash::install_hook` replaces the panic
  hook. The pure parts — `crash_log_path` (XDG state dir rules),
  `format_report`, `panic_message` (a payload is `&str` or `String`) and
  `build_id`, which walks the ELF program headers to the `PT_NOTE`
  segments and the `GNU`/type 3 note with checked arithmetic — are unit
  tested on synthetic ELF bytes. `panic = "abort"` stays: the hook runs
  before the abort.
- **Split symbols.** The `dist` profile is `release` with debug info and
  no strip; `build.sh` runs `objcopy --only-keep-debug`, then
  `--strip-all --add-gnu-debuglink`, and `check_debug_symbols` confirms
  both files have the same build id and the binary stays under 2 MB.
- **One staging tree.** The stripped binary, desktop entry, icons,
  metainfo and licenses are laid out under `target/package/stage/usr`;
  the tarball, `cargo deb`, `cargo generate-rpm` and the Flatpak manifest
  all package that same tree, so they cannot drift apart.

## Try it

1. Run `env -u DISPLAY -u WAYLAND_DISPLAY cargo run`: the preflight
   message instead of a window. Add `DRAW_SKIP_PREFLIGHT=1` and
   `XDG_STATE_HOME=/tmp/s`: miniquad's panic, now as a report in
   `/tmp/s/draw/crash.log` with the build id of
   `readelf -n target/debug/draw`.
2. Add a fake `libfoo.so.1 required libfoo libfoo1` line to
   `packaging/runtime-libs` and run `cargo test --test packaging`: the
   `.deb` and `.rpm` dependency tests name the drift.
3. Build on the host with `cargo build --profile dist` and run
   `objdump -T target/dist/draw | grep -o 'GLIBC_[0-9.]*' | sort -uV |
   tail -1`: the version your machine would force on every user.

## Further reading

- `man ld.so` — library search order and `/etc/ld.so.cache`.
- Ulrich Drepper, *How To Write Shared Libraries* — symbol versioning.
- The ELF specification, *Note Section*; `readelf -n`.
- XDG Base Directory Specification — `XDG_STATE_HOME`.
