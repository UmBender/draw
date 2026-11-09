---
id: ADR-T83-1
title: Paste text through an input event
status: accepted
kind: decision
date: 2026-11-09
task: T83
builds_on: [ADR-T29-2, ADR-T35-1, ADR-T75-1, ADR-T12-1]
amends: [ADR-T29-2]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T83-1 Paste text through an input event

## Context

`Ctrl+V` in an open text block should insert the system clipboard's text
([[T83 Paste text from the clipboard]]). `core` knows nothing about windows
and the clipboard only exists in the running app. In edit mode no binding
fires and `Ctrl+V` did nothing ([[ADR-T29-2 Text edit mode]]); outside it
`Ctrl+V` pastes shapes from the internal clipboard.

The vendored `window::clipboard_get` holds the display mutex while the
Wayland backend runs `wl_display_roundtrip`. That round trip dispatches
whatever is queued, and several handlers take the same mutex
(`wl_pointer.motion`, the tablet tool's motion, touch, `wl_output.scale`
with `try_lock().unwrap()`). `Ctrl+V` with the mouse moving would hang or
panic the app.

## Decision

- **Event.** `InputEvent::Paste(String)` carries the text. `InputEvent`
  stops being `Copy`; it stays `Clone`.
- **The shell reads the clipboard** only for a press of exactly `Ctrl+V`
  while a text edit is open (`input_map::wants_clipboard`), and sends
  `Paste` instead of the key. Every other `Ctrl+V` goes on as a key, so
  shapes still paste, and the system clipboard is not touched for them.
  An empty or unavailable clipboard sends nothing.
- **Core cleans the text** (`text::clean_paste`): `\r\n` and `\r` become
  `\n`, other control characters are dropped, a tab becomes
  [`TAB_SPACES`] spaces, and the result stops at 64 KiB
  (`PASTE_MAX_BYTES`) on a whole character. The limit counts the cleaned
  bytes, because those are what the block holds. `Paste` with no open edit
  does nothing.
- **Undo.** Pasted text belongs to the edit session, so `Ctrl+Z` during
  the edit discards it and a finished edit stays one history entry.
- **Line breaks in a grid cell are kept**, although the `Enter` key walks
  to the next cell there: the key is a navigation choice, the pasted text
  is content.
- **Vendored patch.** `window::clipboard_get` takes the clipboard out of
  the display data and releases the mutex before reading, so handlers run
  by the round trip can lock it. `clipboard_set` is unchanged. Marked
  `draw patch (ADR-T83-1)`.
- **Wake-up.** The round trip may queue input that the blocking loop would
  not notice ([[ADR-T12-1 Blocking event loop with cached frame]]), so the
  shell calls `schedule_update` after each read.
- **End-to-end.** The harness places text on the clipboard with `wl-copy`
  when it is installed, otherwise it hands the text to the app through
  `DRAW_E2E_CLIPBOARD`, which calls `clipboard_set`. Inert unless the suite
  sets it ([[ADR-T75-1 Headless sway session for end-to-end tests]]).

## Alternatives considered

- **`Editor::paste_text(&str)`, keeping `InputEvent` `Copy`** — a second
  way into the editor that the fuzzer's event stream would not reach, and
  `tests/fuzz.rs` must exercise every input.
- **The shell cleans the text** — untestable there, and core could not
  guarantee what a block holds.
- **Replace the round trip by a flush in the vendored read** — no handlers
  would run, but a clipboard owned by this same app could never answer,
  and drag and drop shares that function.
- **Add a binding to the keymap** — edit mode bypasses the keymap by
  design.

## Consequences

- Events are moved or cloned rather than copied; `TapFilter` clones only
  pointer events, which is free.
- The read blocks the frame until the clipboard's owner has written its
  text, so an owner that hangs stalls the app. Normal owners answer at
  once.
- The backends read `UTF8_STRING` only: an application that does not offer
  it pastes nothing.
- Several pastes can make a block larger than 64 KiB; the limit is per
  paste.

## Rollback plan

Remove `InputEvent::Paste`, the `wants_clipboard` branch in `shell::app`,
`clean_paste` and the hook with `revert(T83)` commits and a rollback ADR.
The vendored patch can stay: it only changes who holds the mutex.
