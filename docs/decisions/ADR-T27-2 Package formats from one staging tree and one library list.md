---
id: ADR-T27-2
title: Package formats from one staging tree and one library list
status: accepted
kind: decision
date: 2026-10-15
task: T27
builds_on: [ADR-0002, ADR-0014, ADR-T12-1, ADR-T27-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T27-2 Package formats from one staging tree and one library list

## Context

[[T27 Linux packaging]] ships a tarball, a `.deb`, an `.rpm` and a local
Flatpak bundle. miniquad loads its window-system libraries with
`dlopen`, so the binary links only libc, libm and libgcc and package
tools that infer dependencies from `NEEDED` entries miss everything the
window needs. Each format must install the same desktop entry, icons
and metainfo, and a launcher must match the running window to its
entry.

## Decision

1. **One library list.** `packaging/runtime-libs` has one line per
   soname: role (`required`; `gl` — at least one of the group;
   `unused` — named in the binary but not loaded by the chosen
   backend), Fedora/RHEL package and Debian/Ubuntu package. The `.deb`
   `depends` and `.rpm` `requires` in `Cargo.toml` are the `required`
   and `gl` packages; tests fail when they drift from the list or when
   the binary names a library the list lacks. The startup preflight
   embeds the same file.
2. **One staging tree.** The container build writes the binary and the
   desktop files under `target/package/stage/` in the final layout
   (`usr/bin`, `usr/share/applications`, `usr/share/icons/hicolor/…`,
   `usr/share/metainfo`, `usr/share/licenses/draw`). The tarball,
   `cargo-deb`, `cargo-generate-rpm` and the Flatpak manifest all read
   that tree.
3. **Tools.** `cargo-deb` and `cargo-generate-rpm`, pinned and
   installed in the build image of
   [[ADR-T27-1 Build in an AlmaLinux 8 container for a glibc 2.28 floor]];
   both are build-time tools, the app keeps its dependencies. Flatpak
   uses `flatpak-builder` on the host with a manifest that installs the
   staged files (no build from source inside the SDK), on
   `org.freedesktop.Platform` 26.08.
4. **Application id** `io.github.UmBender.draw` names the desktop file,
   the icon and the metainfo, as Flatpak and AppStream require and as a
   future native Wayland `app_id` will need. The X11 WM class is `draw`
   and the desktop entry says `StartupWMClass=draw`.
5. **Tarball install.** `install.sh` copies the staged `usr/` tree into
   a prefix (default `~/.local`), rewriting `Exec=` to the absolute
   binary path when the prefix is not `/usr`; `--uninstall` removes the
   same fixed list of files.

## Alternatives considered

- **`$auto` dependencies** — `cargo-deb`'s `$auto` reads `NEEDED`, which
  misses every `dlopen`ed library: the bug this task fixes.
- **`rpmbuild` and `dpkg-deb` with hand-written spec/control files** —
  two more formats to keep in sync; `dpkg-deb` is not in AlmaLinux 8.
- **Flatpak built from source in the SDK** — needs the Rust SDK
  extension and vendored crates (`flatpak-cargo-generator`); worth it
  only for a Flathub submission, which is out of scope.
- **`draw.desktop` as the file name** — fine for `.deb` / `.rpm` but not
  accepted by Flatpak or AppStream; one name for all formats is simpler.

## Consequences

- A new `dlopen` from a macroquad upgrade fails a test instead of
  shipping a package that cannot open a window.
- Changing a dependency means editing `runtime-libs` and `Cargo.toml`
  together; the test names both.
- [[T28 Experimental native Wayland backend]] flips the Wayland lines
  from `unused` to a role and the preflight follows.

## Rollback plan

Drop a format by removing its step from `scripts/package.sh` and its
metadata; the staging tree and library list stay for the others.
