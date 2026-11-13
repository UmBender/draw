---
id: ADR-T162-1
title: RawEvent carries the backend records and the pen samples
status: accepted
kind: decision
date: 2026-11-13
task: T162
builds_on: [ADR-T140-1]
amends: [ADR-T140-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T162-1 RawEvent carries the backend records and the pen samples

## Context

[[ADR-T140-1 The Gpu and Platform traits]] gave `RawEvent` the keys,
characters, pointer, wheel, tablet, resize, focus and seat events. The
shell also reads, from the patched window library, the Wayland diagnostics
records (ADR-T47-1), the pure helpers `device_change` (ADR-T44-1) and
`pointer_shadowed_by_pen` (ADR-T40-1), and the pen samples (ADR-T35-1).
The flight recorder needs the records in order with the input, and the pen
pairing must stay lossless.

## Decision

- `RawEvent` gets `Backend(BackendRecord)`. `BackendRecord`, `BackendEvent`
  and `Device` are copied into `draw-sys` unchanged (`at: Instant` and the
  event), so the records travel in the same ordered stream as the input,
  and a platform without such records never sends the variant.
- `DeviceChange`, `device_change`, `has_release_request` and
  `pointer_shadowed_by_pen` are copied into `draw-sys` as pure functions
  with the same behaviour and tests.
- Pen pairing: `RawEvent::Tablet { x, y, pressure }` is a sample. The
  collector queues it and pairs it with the next pointer event at the same
  spot, as `set_pen_samples` did; `set_pen_samples` stays for callers that
  hand over a frame's samples at once (it clears the queue first, which is
  the frame boundary). The macroquad translation sends the frame's samples
  before the frame's other events, like the collector had them before.
- `RawKey::Other(code)` carries miniquad's key code (an X11 keysym) so the
  flight recorder's privacy rule (`key_tag`) is lossless: text keys are the
  letters, digits, space, brackets and the `Other` codes in the same
  keysym ranges as before. `KeyTag::Named` holds a `RawKey`; the dump shows
  its `Debug` form (`Escape`, `Letter('Z')`, `Other(44)`), which changes the
  names of letters, digits and unlisted keys in dumps only.
- Keypad Enter maps to `RawKey::Enter` (the editor treated both alike).
- The only module that names miniquad or macroquad input types is
  `src/shell/platform_mq.rs`. It also holds the test helpers that build
  `RawEvent`s with miniquad's key code names.

## Consequences

- `Collector` no longer implements `EventHandler`; it has `handle(RawEvent)`
  and the shell feeds it through `platform_mq::MqInput`.
- `vendor/miniquad` is unchanged.
