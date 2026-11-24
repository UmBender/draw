---
id: T191
title: Windows pen and clipboard
status: todo
wave: 80
branch: task/T191-windows-pen-and-clipboard
depends_on: [T190]
adrs: []
feature: "[[Windows]]"
tutorial: ""
tags: [task, windows]
---

# T191 Windows pen and clipboard

## Goal

On Windows, a pen draws with pressure and keeps every sample, and the
clipboard reads and writes text and writes images that other programs
can paste.

## Known facts (2026-11-24)

- Decisions: [[Windows port proposal]] §3.5 (pen) and §3.7 (clipboard).
- Pen: `WM_POINTER*` with `GetPointerPenInfoHistory` returns the samples
  Windows coalesced between messages, oldest last; pressure is
  `0..=1024`. `RawEvent::Tablet` takes pressure in `0..=1`. The tap rule
  of [[T34 Pen tablet input]] stays in the shell.
- `src/shell/tablet.rs` prints the tablet found at start-up
  (`--tablet`); on Windows the pen devices come from `GetPointerDevices`.
- **Nobody can test a real pen on Windows** (owner's decision 1,
  2026-11-24): CI injects pen input with
  `CreateSyntheticPointerDevice(PT_PEN)` and
  `InjectSyntheticPointerInput` (Windows 10 1809 and later).
- Clipboard: `CF_UNICODETEXT` both ways; images as the registered `PNG`
  format and `CF_DIBV5` (with alpha). `OpenClipboard` is retried briefly
  when another program holds it.
- Windows tests run only on the Windows CI job (proposal, question 7).

## Spec

Draft; the spec step names the final tests.

- **AC-1** — Pen samples become `Tablet` events with pressure in
  `0..=1`, every coalesced sample kept, oldest first.
  Tests (pure): `pen_pressure_1024_maps_to_one`,
  `pen_history_is_reported_oldest_first`; (Windows)
  `win32_synthetic_pen_stroke_reports_pressure_and_history`.
- **AC-2** — Pen devices are listed for the start-up message.
  Test (Windows): `win32_synthetic_pen_device_is_listed`.
- **AC-3** — Text round trip through the clipboard, with non-ASCII
  characters. Test (Windows): `win32_clipboard_text_round_trip`.
- **AC-4** — A copied image is readable as `PNG` and as `CF_DIBV5`,
  alpha kept, rows in the right order.
  Tests: `dibv5_from_rgba_keeps_alpha_and_row_order` (pure),
  `win32_clipboard_image_is_png_and_dib` (Windows).
- **AC-5** — A clipboard held by another program is retried, then
  reported, never a panic. Test (Windows):
  `win32_busy_clipboard_is_retried_then_reported`.

Manual check (the owner's friend): copy a selection as an image and
paste it in Paint and in a browser; copy and paste text both ways.

## Out of scope

- Pasting images into draw (draw pastes text only).
- WinTab tablets: Windows Ink only.

## Files owned

- `crates/draw-sys/src/win32/pen.rs`, `crates/draw-sys/src/win32/clipboard.rs` (new),
  `crates/draw-sys/src/win32/decode.rs`, `crates/draw-sys/src/win32/window.rs`,
  `crates/draw-sys/src/win32/platform.rs`
- `crates/draw-sys/Cargo.toml` (`windows` features), `Cargo.lock`
- `crates/draw-sys/tests/win32_pen.rs`, `crates/draw-sys/tests/win32_clipboard.rs`,
  `crates/draw-sys/tests/win32_pure.rs`
- `src/shell/tablet.rs` (the Windows pen listing)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T191): specify the Windows pen and clipboard`
- [ ] tests — `test(T191): add failing tests for the Windows pen and clipboard`
- [ ] models — `feat(T191): add the Windows pen and clipboard types`
- [ ] behaviour — `feat(T191): implement the Windows pen and clipboard`
- [ ] quality — `chore(T191): pass clippy and rustfmt`

## Log

- 2026-11-24 — written from the Windows port proposal.
