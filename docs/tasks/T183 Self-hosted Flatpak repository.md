---
id: T183
title: Self-hosted Flatpak repository
status: todo
wave: 75
branch: task/T183-self-hosted-flatpak-repo
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, packaging]
---

# T183 Self-hosted Flatpak repository

## Goal

Users can install and update `draw` via Flatpak (`flatpak remote-add` + `flatpak install` + `flatpak update`) or a single-file `.flatpak` bundle from GitHub Releases without waiting for Flathub review.

## Known facts (2026-11-18)

- `packaging/flatpak/io.github.UmBender.draw.yml` and `scripts/package.sh` already produce a working `.flatpak` bundle (`dist/draw-1.6.0-x86_64.flatpak`).
- A static OSTree repository plus a `.flatpakrepo` descriptor hosted on GitHub Pages (or release assets) gives users `flatpak update` support with zero third-party approval.

## Spec

- **AC-1** — `scripts/package.sh` (or `scripts/flatpak-repo.sh`) exports the Flatpak build into an OSTree repo (`flatpak build-export`) and generates a ready-to-host `draw.flatpakrepo` file pointing to the static repo URL.
- **AC-2** — Local verification installs from the generated OSTree repo (`flatpak remote-add --no-gpg-verify` or GPG-signed) and runs `flatpak run io.github.UmBender.draw --version`.
- **AC-3** — `README.md`: an "Install via Flatpak" section covering both the direct `.flatpak` release bundle and the self-hosted `.flatpakrepo` remote.

## Out of scope

- Submitting `io.github.UmBender.draw` to Flathub (`flathub/flathub` PR review).

## Files owned

- `packaging/flatpak/draw.flatpakrepo` (template/descriptor)
- `scripts/package.sh` / `scripts/flatpak-repo.sh`
- `README.md` (Flatpak install section)
- this note

## Subtasks

- [ ] behaviour — `feat(T183): export OSTree Flatpak repo and .flatpakrepo descriptor`
- [ ] docs — `docs(T183): document Flatpak bundle and remote installation`
- [ ] publish — host static OSTree repo (e.g., GitHub Pages) and attach `.flatpak` bundle to releases
