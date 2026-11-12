---
id: ADR-T117-1
title: Scored shape recognition with looser limits
status: accepted
kind: decision
date: 2026-11-12
task: T117
builds_on: ["[[ADR-T109-1 Shape recognition on Z]]"]
amends: ["[[ADR-T109-1 Shape recognition on Z]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T117-1 Scored shape recognition with looser limits

## Context

After trying 1.5.0 the owner found `Z` too sensitive: it is used to be fast, and a hurried stroke often stayed as drawn because [[ADR-T109-1 Shape recognition on Z]] is a veto chain, one limit failing narrowly meaning no fit. The hard rule stays: a stroke that is clearly not a line, box or circle must stay a stroke.

## Decision

The fitter of ADR-T109-1 keeps its steps and changes these limits and rules; the constants are in `core::recognize`.

- **Line**: chord at least `0.75` of the length (was `0.8`), every sample within `0.1` of the chord (was `0.05`).
- **Closed**: the ends at most `0.3` of the length apart (was `0.15`); a loop that overshoots its start is closed too, since the gap is measured end to start. Winding at least `0.7` of a full turn (was `0.8`).
- **Scoring instead of vetoes.** Each closed candidate gets a score: the larger of its mean residual over `0.05` and its largest residual over `0.14` (both of the box diagonal), where the largest residual leaves out the worst `4 %` of the samples (a short tail or flick). A score of `1` or less passes.
- **Corners as plausibility.** A rectangle candidate needs at least 3 of the 4 box corners within `0.12` of the diagonal; an ellipse candidate at most 1 corner within `0.06`.
- **Ambiguity by fit.** When both candidates pass, the one with the score below `0.9` of the other's wins; otherwise the stroke stays as drawn.
- Unchanged: the 128 samples, the minimum-area box, the axis-aligned result, one undo step, colour and width kept, `Z` only right after a stroke.

Measured tolerance (noise level at which a family first fails, `1.0` being the level it must survive; the test `tolerance_table_matches_the_adr` pins the same numbers): hurried circle 2.9, circle that overshoots 4.4, circle that stops short 3.6, circle with a tail 4.3, squashed ellipse 1.5, lopsided rectangle 2.6, rectangle that stops short 4.6, rectangle that overshoots 3.3, rounded rectangle 3.8, wobbly line 1.6, curved line 1.1.

## Alternatives considered

- **Only loosen the constants** — still a veto chain, so a near-miss on one limit fails the whole stroke.
- **A second, looser pass** — doubles the code and makes two thresholds to keep in step.

## Consequences

- Hurried circles, rectangles with rounded or missing corners, wobbly lines and strokes with a tail are recognised. A hexagon may now read as a circle: it is within the new residual limit.
- A wrong guess is still possible near the limits; the owner can ask for a setting after trying it.

## Rollback plan

Restore the constants of ADR-T109-1 and the corner/residual veto in `closed_kind`; this ADR is then reverted by a new ADR.
