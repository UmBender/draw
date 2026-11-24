---
id: T186
title: CRLF line endings in draw files
status: ready
wave: 77
branch: fix/draw-file-crlf
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, bug, windows]
---

# T186 CRLF line endings in draw files

## Goal

A `.draw` file or a `.snap` snapshot whose lines end in CRLF (a file
that went through Windows, Git's `autocrlf` or an editor) opens like
the same file with LF endings. draw keeps writing LF.

## Known facts (2026-11-24)

- Found while writing the [[Windows port proposal]] (§3.11), by reading
  the code; not yet seen in the app.
- `snapshot::Reader::new` (`crates/draw-core/src/core/snapshot.rs`)
  splits the text on `'\n'` only, and records split their tokens on
  `' '`. With CRLF the last token of every line keeps a `\r`: the
  header `draw-file 1` reads the version `1\r`, which is not a number,
  so `DocFile::parse` fails on the first line.
- `--from-md` already normalises `\r\n` and `\r`
  (`markdown.rs`, `source.replace("\r\n", "\n")`): only the reader of
  `.draw` and `.snap` text is affected.
- Text inside shapes is quoted on one line, with `\r` and `\n` escaped
  (`snapshot::quote`), so dropping the
  `\r` before each `\n` cannot change a text's content.
- Independent of the rest of the Windows port: it fixes files moved
  between Linux machines through a Windows checkout too.

## Spec

- **AC-1** — Regression: `tests/files/v1.draw` with every `\n` turned into
  `\r\n` parses to the same document as the LF file.
  Test: `regression_T186_crlf_draw_file_opens` (`tests/files.rs`; the
  CRLF text is built in the test, so no CRLF fixture is checked in).
- **AC-2** — A `.snap` snapshot with CRLF endings loads like the LF one.
  Test: `snapshot_with_crlf_endings_loads` (`snapshot.rs` unit test).
- **AC-3** — Only a `\r` right before a `\n` (or at the end of the text)
  is dropped; a `\r` elsewhere is still an error naming its line.
  Test: `lone_carriage_return_inside_a_line_is_an_error`.
- **AC-4** — Writers emit `\n` only.
  Test: `draw_file_text_has_no_carriage_return`.

## Out of scope

- `.gitattributes` and the Windows CI job ([[T187 Windows ADRs and portable seams]]).
- Old Mac `\r`-only endings.

## Files owned

- `crates/draw-core/src/core/snapshot.rs` (`Reader::new` and its tests)
- `crates/draw-core/src/core/file.rs` (tests only)
- `tests/files.rs`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T186): specify CRLF acceptance criteria`
- [ ] regression — `test(T186): add regression test for CRLF draw files`
- [ ] behaviour — `fix(T186): accept CRLF line endings in draw files`
- [ ] quality — `chore(T186): pass clippy and rustfmt`
- [ ] docs — `docs(T186): update the notes`

No models step: no new type.

## Log

- 2026-11-24 — written from the Windows port proposal (owner's request).
