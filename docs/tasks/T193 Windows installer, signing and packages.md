---
id: T193
title: Windows installer, signing and packages
status: todo
wave: 82
branch: task/T193-windows-installer-signing-and-packages
depends_on: [T192]
adrs: []
feature: "[[Windows]]"
tutorial: ""
tags: [task, windows]
---

# T193 Windows installer, signing and packages

## Goal

A tagged release publishes, for Windows, a signed per-user MSI that
associates `.draw` files, a signed portable zip, and the winget and
Scoop manifests. The build is hardened and needs no runtime to install.

## Known facts (2026-11-24)

- Owner's decisions (2026-11-24, [[Windows port proposal]] §8a):
  installer with the `.draw` association from the first release; signed
  from the first release; `x86_64` only, ARM later.
- Installer (proposal §5): a per-user MSI built with WiX that installs to
  `%LOCALAPPDATA%\Programs\draw` with no administrator prompt, adds the
  Start menu entry, registers a ProgId under `HKCU\Software\Classes` with
  the icon and the `open` verb `"draw.exe" "%1"`, and uninstalls
  cleanly. `draw FILE.draw` already opens a file. WiX's licence terms
  are checked in the spec step.
- **Signing** (checked 2026-11-24): Azure Artifact Signing takes
  individual developers only in the USA and Canada, so not the owner.
  **SignPath Foundation** (proposed) signs open-source projects for
  free from GitHub Actions; Windows then shows SignPath Foundation as
  the publisher; it asks for MFA on GitHub and SignPath and an OSI
  licence (draw is MIT). Fallback: an OV certificate from a commercial
  CA with cloud signing, which shows the owner's name, for a yearly fee.
  **Owner step before this task can finish**: apply to SignPath
  Foundation (or buy the OV certificate) and add the secrets to the
  repository. Signing runs only in the release workflow.
- Hardening (proposal §3.10): static CRT (`+crt-static`), Control Flow
  Guard, `/CETCOMPAT`, for the Windows target in `.cargo/config.toml`.
  The release profile stays (fat LTO, `panic = "abort"`).
- The PDB is kept as a CI artifact of the release, not in the packages,
  like `draw.debug` on Linux.
- Icon: an `.ico` made from the existing artwork
  (`packaging/` icons) and a `.rc` compiled by `rc.exe` from `build.rs`
  when building on Windows.
- The binary limit of [[ADR-T82-1 Shipped binary limit of 20 MiB]]
  applies to `draw.exe`.

## Spec

Draft; the spec step names the final tests.

- **AC-1** — `draw.exe` is hardened and self-contained: CFG and CET
  flags in the PE headers, no `vcruntime`/`ucrtbase` import, under
  20 MiB. Tests: `release_exe_has_cfg_and_cet_compat`,
  `release_exe_imports_only_system_dlls`, `release_exe_is_under_the_limit`
  (PE parsing in `tests/packaging.rs`, run on the Windows release job).
- **AC-2** — The MSI installs per user without an administrator,
  registers the association and the Start menu entry, and uninstall
  removes them. Test (Windows): `msi_installs_and_uninstalls_per_user`.
- **AC-3** — Opening a `.draw` file through the shell starts `draw.exe`
  with that file. Test (Windows): `draw_file_association_opens_draw`.
- **AC-4** — `draw.exe` and the MSI carry a valid signature in a release
  build. Test: `release_artifacts_are_signed` (`signtool verify /pa` in
  the release workflow).
- **AC-5** — The winget and Scoop manifests name the release's version,
  URLs and hashes. Test: `windows_manifests_match_the_release`
  (`tests/packaging.rs`).
- **AC-6** — `draw.exe` has the icon and a version resource with the
  crate version. Test: `release_exe_has_icon_and_version`.

Manual check (the owner's friend): install the MSI, open a `.draw` file
by double click, uninstall; note what SmartScreen shows.

## Out of scope

- Windows on ARM (owner's decision 5).
- The Microsoft Store and MSIX.
- Automatic updates.

## Files owned

- `packaging/windows/**` (WiX source, `.ico`, `.rc`, winget and Scoop manifests)
- `build.rs` (the icon and version resource), `.cargo/config.toml` (new)
- `.github/workflows/release-windows.yml` (new), `scripts/release.sh`
- `tests/packaging.rs`
- `README.md` (Windows install section)
- `docs/features/Windows.md` (install section), `docs/features/Linux packages.md`
  (link to the Windows packages)
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T193): specify the Windows packages`
- [ ] tests — `test(T193): add failing tests for the Windows packages`
- [ ] models — `feat(T193): add the installer and the manifests`
- [ ] behaviour — `feat(T193): build, sign and publish the Windows packages`
- [ ] quality — `chore(T193): pass clippy and rustfmt`
- [ ] docs — `docs(T193): document installing on Windows`

## Log

- 2026-11-24 — written from the Windows port proposal.
