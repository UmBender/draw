---
id: T12
title: App shell and toolbar
status: review
wave: 6
branch: task/T12-app-shell
depends_on: [T07, T09, T10, T11]
adrs: ["[[ADR-0006 Redraw on demand]]", "[[ADR-0012 Fixed palette and theme tokens]]", "[[ADR-T12-1 Blocking event loop with cached frame]]"]
feature: "[[Toolbar]]"
tutorial: "[[12 The app loop, idle redraw and UI]]"
tags: [task]
---

# T12 App shell and toolbar

## Goal

The runnable app: window, input translation, idle-friendly loop, and a minimal
toolbar styled with the theme.

## Spec

Input reaches the shell through a macroquad **input subscriber**
(`input::utils::repeat_all_miniquad_input`), not by polling per-frame state:
every motion sample, press and release arrives in order, so fast pen strokes and
clicks shorter than a frame are never lost. Event positions are physical pixels;
the shell divides by the DPI scale to get the logical pixels the editor and
renderer use. The loop and the redraw strategy are recorded in
[[ADR-T12-1 Blocking event loop with cached frame]].

Pure, unit-tested:

- **AC-1** — `input_map`: `map_key(KeyCode) -> Option<Key>` covers `A`–`Z`,
  `0`–`9`, `Delete`, `Backspace`, `Escape`, `Space`, `Tab`, `[`, `]` and
  nothing else; `map_button(MouseButton) -> Option<PointerButton>` (unknown →
  `None`); `map_mods(KeyMods) -> Modifiers` (the logo key is dropped);
  `to_logical(x, y, dpi)` divides by a DPI scale that falls back to `1` when it
  is not finite and positive.
  *Tests:* `input_map::tests::map_key_letters_and_digits`,
  `map_key_named_keys`, `map_key_unbound_is_none`, `map_button_all`,
  `map_mods_drops_logo`, `to_logical_divides_by_dpi`,
  `to_logical_bad_dpi_is_identity`.
- **AC-1b** — `input_map::Collector` (a `miniquad::EventHandler`) turns raw
  events into `InputEvent`s in arrival order: motion → `PointerMove`, buttons →
  `PointerDown`/`PointerUp` at the event position, wheel `y` → `Scroll` at the
  last pointer position, key down (also auto-repeat) / up → `KeyDown`/`KeyUp`
  with the event's modifiers; unmapped keys and buttons produce nothing.
  Pointer events carry the modifiers of the modifier keys currently held
  (tracked from key down/up). *Tests:* `collector_translates_in_order`,
  `collector_scroll_uses_last_pointer`, `collector_skips_unmapped`,
  `collector_pointer_mods_follow_modifier_keys`, `collector_scales_by_dpi`,
  `collector_key_events_carry_event_mods`.
- **AC-2** — `toolbar::layout(viewport) -> Vec<Button>`: one button per tool (9,
  in keymap order), 6 colour swatches, then undo and redo — 17 buttons of
  32 × 32 px stacked in a vertical strip at the left edge, without overlap,
  groups separated by a larger gap. `toolbar::panel(viewport)` is the strip
  background: full viewport height, as wide as a button plus padding.
  *Tests:* `toolbar::tests::layout_has_every_button_in_order`,
  `layout_buttons_are_32px_in_left_strip`, `layout_buttons_do_not_overlap`,
  `layout_buttons_inside_panel`.
- **AC-3** — `toolbar::hit(buttons, pos) -> Option<Command>`: the command of
  the button under `pos` (`SetTool`, `SetColor`, `Undo`, `Redo`), `None`
  elsewhere or for non-finite `pos`. `toolbar::route(visible, viewport, event)`
  decides where an event goes: a pointer press on the visible panel becomes the
  hit command (or is swallowed between buttons) and never reaches the canvas;
  everything else — moves, releases, scrolls, keys, presses when hidden — is
  forwarded. *Tests:* `hit_each_button_returns_its_command`,
  `hit_outside_buttons_is_none`, `hit_non_finite_is_none`,
  `toolbar_click_not_forwarded`, `route_press_between_buttons_is_swallowed`,
  `route_hidden_toolbar_forwards`, `route_canvas_and_other_events_forward`.

Shell (manual checklist in the task log):

- **AC-4** — Loop: collect events → `Editor::handle`/`Editor::apply` → re-render
  the scene only when dirty ([[ADR-0006 Redraw on demand]]). Strategy recorded in
  [[ADR-T12-1 Blocking event loop with cached frame]].
- **AC-5** — Toolbar shows the active tool and colour with the `accent` token;
  undo/redo are dimmed when unavailable; hidden with `Tab`.
- **AC-6** — Runs on Wayland (native or XWayland — record which) without input
  loss; idle CPU ≈ 0 % (measure with `top`, record the number).

## Out of scope

Menus, settings UI, saving.

## Files owned

`src/shell/app.rs`, `src/shell/input_map.rs`, `src/shell/toolbar.rs`,
new ADR `docs/decisions/ADR-T12-1 *.md`.

## Subtasks (one commit each)

- [x] spec — `docs(T12): specify app shell and toolbar acceptance criteria` (e029e71)
- [x] tests — `test(T12): add failing tests for input mapping and toolbar` (0825eec)
- [x] models — `feat(T12): add input collector and toolbar types` (945fe43)
- [x] behaviour — `feat(T12): implement input collection, toolbar and idle-friendly loop` (97b09e6)
- [x] quality — `refactor(T12): pass clippy and rustfmt` (28dcf17)
- [x] docs — `docs(T12): add feature note and tutorial`

## Learning path

Step 12 — requires steps 7–11.

## Log

- 2026-10-01 — Spec refined. Reading macroquad 0.4.16 showed `begin_frame`
  always clears the window, so the blocking loop alone cannot skip drawing;
  recorded blocking loop + cached render target as
  [[ADR-T12-1 Blocking event loop with cached frame]]. Input comes from a
  macroquad input subscriber so no motion sample or short click is lost.
- Tests: 25 unit tests in `input_map` and `toolbar`; red on missing items.
- Behaviour: `window_conf` now returns `WindowSettings` (same field names)
  converted `Into` macroquad's `Conf`. This keeps `main.rs` and the T00-owned
  `tests/skeleton.rs` unchanged; a plain miniquad `Conf` would have left
  `update_on` empty and the blocking loop would never wake on input.
- Quality: clippy asked for a bitmask instead of six bools in `HeldMods`.
  `scripts/check.sh` green.
- Manual checklist (Fedora 44, KDE Plasma Wayland session, release build):
  - AC-4 — the loop blocks while idle; the scene renders correctly into the
    cached target (screenshot: canvas `bg`, toolbar strip, not flipped).
  - AC-5 — toolbar shows tool letters and swatches; active tool (pen) and
    colour (ink) outlined in `accent`; undo/redo dimmed on a fresh document.
    `Tab` hiding still to be confirmed by hand.
  - AC-6 — runs through **XWayland** (miniquad `X11Only` default). Idle CPU
    measured with `top`: **0.0 %** (0.15 s CPU total over ~12 s, state `S`).
    Drawing without input loss still to be confirmed by hand.
