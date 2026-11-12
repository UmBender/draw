---
title: End-to-end tests
task: "[[T75 End-to-end test suite]]"
adrs: ["[[ADR-T75-1 Headless sway session for end-to-end tests]]", "[[ADR-T75-2 Wheel notches through the virtual pointer]]"]
tutorial: ""
shortcuts: []
tags: [feature, testing]
---

# End-to-end tests

## What it does

Runs the real `draw` binary in a private headless Wayland session and
drives it with real compositor input. This reaches the parts the core
tests cannot: the window and backend, `shell::input_map`, the renderer and
its caches, the toolbar, the log and a clean exit. Nothing reaches the
desktop: the session has its own runtime directory and socket, and the
harness refuses to start any tool otherwise.

## How to use

Needs `sway`, `wtype` and `grim`.

| Action | Command |
|--------|---------|
| Run every case | `scripts/e2e.sh` |
| Run one case | `scripts/e2e.sh text-formula` |
| Rewrite the goldens, then look at them | `scripts/e2e.sh --update` (or `DRAW_UPDATE_SNAPSHOTS=1 scripts/e2e.sh`) |

A failed case keeps `actual.snap`, `actual.ppm`, `diff.ppm` and
`state/draw/draw.log` in `target/e2e/<case>/`. The nightly workflow runs
the suite and uploads that directory on failure.

## Writing a case

A case is a directory `tests/e2e/<case>/` with an `input` file, one
directive per line:

| Directive | Meaning |
|-----------|---------|
| `scene <name>` | generated start state, instead of `start.snap` |
| `sections <name>...` | compare only these snapshot sections |
| `screenshot [pixels channel]` | also compare `expected.ppm` |
| `text <text>` | type literally; may not start with `-` |
| `clipboard <text>` | put text on the system clipboard, with `wl-copy` or else through the app's `--e2e-clipboard` hook; may not start with `-` |
| `key <keysym>` | one key, by XKB name (`Return`, `Escape`) |
| `chord <mod>... <keysym>` | key with modifiers, e.g. `chord ctrl shift z` |
| `hold <keysym> <ms>` | hold a key down |
| `end-keyboard` | end the active keyboard (may send `no_keymap`) |
| `move x y`, `press [button]`, `release [button]` | pointer |
| `click x y`, `drag x1 y1 x2 y2 [segments]` | left button |
| `wheel up`, `wheel down` | one wheel notch |
| `resize w h` | resize the output and wait for the window |
| `checkpoint <name>` | compare the state with `<name>.snap` here |
| `close`, `sigint` | how the case ends (default `close`) |

An optional `start.snap` holds the state to start from. Run
`scripts/e2e.sh --update <case>` and check the new `expected.snap` and
screenshot before committing them. Keep screenshots to cases where pixels
are the point: each one is 1.4 MB.

The comparison ignores a 26×26 box at the pointer's last position,
because whether a capture shows the mouse cursor depends on the compositor
version ([[ADR-T80-1 Screenshots ignore the box under the pointer]]). So a
case that needs the pixels there must move the pointer away first.

Each case ends clean: exit code 0 (or death by `SIGINT` for a `sigint`
case), no `crash.log`, and no `WARN` or `ERROR` line in `draw.log`.

## Hooks in the app

Inert unless their flag is given (hidden from `--help`).

| Flag | Receives |
|------|----------|
| `--e2e-dump <file>` | the editor snapshot, at every sync |
| `--e2e-idle <file>` | the number of syncs answered, after the frame is on screen |

The harness sends `Ctrl+Shift+9` after each input step and waits for the
count, so it never sleeps for a fixed time. A hung app fails the case after
the timeout with its log attached.

## Not covered

Real pen tablet hardware, frame-time budgets, and X11-only behaviour.
