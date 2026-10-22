---
id: T41
title: Integral superscript at the advance
status: done
wave: 27
branch: fix/integral-superscript
depends_on: [T36]
adrs: ["[[ADR-T41-1 Integral superscript at the advance]]"]
feature: "[[TeX formulas]]"
tutorial: "[[36 Operators whose limits sit beside them]]"
tags: [task]
---

# T41 Integral superscript at the advance

## Goal

In `$\int_{0}^{\pi} f(x) \, dx$` the upper limit sits right beside the
top of the sign, as in inline LaTeX. Today (1.4.2) it floats about half
an em to the right of the sign's ink, while the lower limit is fine.

## Known facts (2026-10-22, owner's screenshot of 1.4.2)

- `scripts` (`src/core/tex.rs`) puts a superscript at
  `base.width + base.italic` for every base, integrals included
  (ADR-T36-1: "The superscript starts after the italic correction as for
  any base").
- Latin Modern Math's `∫` (U+222B): advance 0.665 em, ink right edge
  `x_max` 0.609 em, italic correction 0.332 em. The ink ends **inside**
  the advance: in an OpenType `MATH` font the advance of a large
  operator already includes its italic correction, which only says how
  far the subscript moves left under the slant. The same holds for
  `∬ ∭ ∮ ∯`.
- **Root cause.** Adding the correction again puts the superscript
  `(0.665 + 0.332 − 0.609) × 1.3 ≈ 0.50 em` right of the ink instead of
  `(0.665 − 0.609) × 1.3 ≈ 0.07 em`.
- `tex_layout_integral_scripts_sit_beside_the_sign` asserts the wrong
  position (`sign.x + advance + italic`), so the suite is green.
- Italic letters (`𝑓^2`) are right: their advance does not include the
  correction, so they keep `width + italic`.

## Proposal (settle in the spec step)

A new ADR amending ADR-T36-1: on an integral the superscript starts at
the advance; the subscript stays at advance − italic. Other bases keep
`width + italic`.

## Spec

Decision: [[ADR-T41-1 Integral superscript at the advance]] — the
proposal is taken.

- **AC-1** — on every integral (`\int \iint \iiint \oint \oiint`) the
  superscript starts at the enlarged sign's advance, and the subscript
  stays at advance − italic correction:
  `tex_layout_integral_superscript_starts_at_the_advance`, and the
  corrected `tex_layout_integral_scripts_sit_beside_the_sign`.
- **AC-2** — the superscript clears the sign's ink but sits within
  0.1 em of it: `tex_layout_integral_superscript_hugs_the_sign`.
- **AC-3** — italic letters keep the superscript after the correction:
  `tex_layout_superscript_clears_the_italic_correction` (unchanged).
- **AC-4** — owner check: `$\int_{0}^{\pi} f(x) \, dx$` looks like inline
  LaTeX (see *Log*).

## Out of scope

- Display-size integrals, `\limits` / `\nolimits`.
- Script placement on other bases.

## Files owned

- `src/core/tex.rs`
- `docs/features/TeX formulas.md`,
  `docs/tutorials/36 Operators whose limits sit beside them.md`
- new ADR `ADR-T41-*`

## Subtasks (one commit each)

- [x] spec — `docs(T41): specify integral superscript placement`
- [x] tests — `test(T41): add failing tests for the integral superscript`
- [x] models — `feat(T41): give scripts a superscript kern argument`
- [x] behaviour — `fix(T41): put an integral's superscript at its advance`
- [x] quality — skipped: `scripts/check.sh` was green with no changes
- [x] docs — `docs(T41): correct the integral superscript in tutorial 36`

## Learning path

No new tutorial step; correct step 36 (operators whose limits sit beside
them).

## Log

- 2026-10-22 — Created from the owner's report: the upper limit of
  `$\int_{0}^{\pi} f(x) \, dx$` sits too far from the sign.
- 2026-10-22 — Implemented per ADR-T41-1: `scripts` takes the
  superscript's `kern`; integrals pass 0, every other base its italic
  correction. The ink-to-script gap of `\int^b` went from 0.50 em to
  0.07 em. The new hugging test measures from the sign's ink to the
  script's pen position (first draft measured to the script's ink and
  so counted its side bearing). The quality step produced no changes
  and was skipped. Tutorial step 36 corrected. AC-4 awaits the owner
  check.
- Owner check (2026-10-22): `$\int_{0}^{\pi} f(x) \, dx$` now shows the
  upper limit beside the top of the sign, as in inline LaTeX (AC-4).
- 2026-10-22 — Integrated into `main`. Status → done.
