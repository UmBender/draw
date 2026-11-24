---
id: ADR-T185-1
title: Server-side decorations through xdg-decoration
status: accepted
kind: decision
date: 2026-11-24
task: T185
builds_on: ["[[ADR-T168-1 Generated Wayland protocol code and surface type states]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T185-1 Server-side decorations through xdg-decoration

## Context

Since [[T149 Cutover removing miniquad and macroquad]] the own Wayland
backend draws the window. It binds no decoration protocol, so KWin and
wlroots compositors treat the window as client-decorated and show no
title bar or frame. miniquad asked for server-side decorations and, on
compositors without them, used `libdecor` or drew a plain bar.

## Decision

- `draw-sys` binds `zxdg_decoration_manager_v1` when the compositor
  announces it. The generated code comes from
  `protocols/xdg-decoration-unstable-v1.xml`, like the other protocols
  (ADR-T168-1).
- `Connection::create_window` creates the toplevel's
  `zxdg_toplevel_decoration_v1` before the first commit and asks for
  `server_side`. The decoration is destroyed before the toplevel.
- The mode the compositor configures is kept and reported by
  `Window::decoration()`. draw acts the same either way: it never draws
  its own decorations.
- Without the manager (GNOME's Mutter), nothing changes: no title bar.

## Alternatives considered

- **Client-side decorations drawn by draw** — a title bar, buttons, move
  and resize grabs and their own rendering pass; much code for the one
  desktop (GNOME) that lacks server-side decorations. Left for a later
  task if the owner needs GNOME.
- **`libdecor` loaded with `dlopen`** — another runtime library and its
  plugin system (GTK on GNOME), against the small dependency list
  ([[ADR-T127-3 The dependency list]]).
- **`org_kde_kwin_server_decoration_manager`** — KDE only; KWin offers
  `zxdg_decoration_manager_v1` as well.

## Consequences

- KDE Plasma, sway, Hyprland and other wlroots compositors draw the
  window's title bar and frame again.
- GNOME still shows no title bar on native Wayland; `--backend x11`
  (Xwayland) gives one.
- One more generated protocol module to keep in sync with its XML.

## Rollback plan

Drop the manager binding and the decoration object in
`wayland/connection.rs` and `wayland/window.rs` and the generated
module, with a rollback ADR; the window goes back to having no
decorations on every compositor.
