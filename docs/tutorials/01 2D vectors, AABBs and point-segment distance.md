---
title: 2D vectors, AABBs and point-segment distance
step: 1
requires: ["[[00 Project layout, lints and the TDD loop]]"]
feature:
code: ["src/core/geom.rs"]
tags: [tutorial]
---

# 2D vectors, AABBs and point-segment distance

> **Learning path step 1.** Requires: [[00 Project layout, lints and the TDD loop]] ·
> Next: *3 Cameras: world vs screen*, *4 Smoothing & Douglas–Peucker*, *5 Shapes & hit-testing*

## Why this matters here

Every other part of the core speaks in points. The camera turns screen points
into world points, the smoother thins out lists of points, shapes store points
and answer "was I clicked?", the eraser asks "how close is the cursor to this
stroke?". All of that needs three things: a small vector type, a bounding box
for cheap rejection, and the distance from a point to a line segment. They live
in `core::geom` and have no dependencies at all.

## The concept

### Vectors

A `Vec2 { x, y }` is both a *point* (a position) and a *displacement* (an
arrow). The operations are component-wise:

```
a + b = (a.x + b.x, a.y + b.y)        move a point by an arrow
a - b = (a.x - b.x, a.y - b.y)        arrow from b to a
a * s = (a.x * s,   a.y * s)          scale an arrow
a · b = a.x*b.x + a.y*b.y             dot product
|a|   = √(a · a)                      length
```

The dot product is the workhorse: `a · b = |a| |b| cos θ`. It is zero for
perpendicular arrows and, when `b` has length 1, it is the length of `a`'s
shadow on `b`. `length_sq` skips the square root, which is enough whenever you
only *compare* distances.

`lerp(a, b, t) = a + (b − a)·t` walks along the line from `a` (`t = 0`) to
`b` (`t = 1`); the box centre is just `lerp(min, max, 0.5)`.

### Axis-aligned bounding boxes

An AABB is the smallest rectangle with sides parallel to the axes that holds a
shape, stored as two corners `min` and `max`. Two boxes overlap exactly when
their x-intervals overlap *and* their y-intervals overlap:

```
a.min.x <= b.max.x && b.min.x <= a.max.x      (same for y)
```

That is four comparisons, so a hit-test or a cull can throw away almost every
shape before doing any real geometry.

### Distance from a point to a segment

```
            p
            |
            | d
            |
  a ------- q ----------- b
```

Project `p` onto the infinite line through `a` and `b`:

```
t = ((p − a) · (b − a)) / |b − a|²
```

`t` is where the foot of the perpendicular falls: `t = 0` at `a`, `t = 1` at
`b`. For a *segment*, clamp `t` to `[0, 1]`, take `q = a + (b − a)·t` and the
answer is `|p − q|`. When `p` is "behind" `a` the clamp snaps `q` to `a`,
when it is past `b` it snaps to `b`.

The trap: if `a` and `b` are the same point, `|b − a|²` is zero and `t` is
`0/0 = NaN`. The segment is then a point, so the answer is simply `|p − a|`.

### Floats are not real numbers

`0.1 + 0.2` is not `0.3` in `f32`, and `NaN` is not equal to itself. The crate
denies clippy's `float_cmp`, so floats are never compared with `==`. Instead:

```
approx_eq(a, b, eps)  ⇔  |a − b| <= eps   (false if anything is NaN/∞)
```

Non-finite numbers also need a policy, because a single NaN in a stored point
poisons every bounding box and distance it touches. Here the rule is: reject
explicitly at the boundary (`sanitize`, `from_points`), never panic.

## How draw implements it

All in `src/core/geom.rs`:

- `approx_eq` — `eps.is_finite() && (a - b).abs() <= eps`. With an infinite
  operand the difference is `∞` or `NaN`, so the comparison is `false`.
- `Vec2` — `Copy`, so it is passed by value everywhere. Operators come from
  `std::ops` (`Add`, `Sub`, `Mul<f32>`, `Div<f32>`, `Neg`, `AddAssign`,
  `SubAssign`); methods `length`, `length_sq`, `dot`, `distance`, `lerp`,
  `approx_eq`, `is_finite`, `sanitize`. Every pure method is `#[must_use]`, so
  writing `v.lerp(w, 0.5);` and discarding the result is a warning.
- `Vec2::sanitize` returns `Option<Vec2>` (`self.is_finite().then_some(self)`).
  Later modules use it on raw input coordinates: `None` means "drop this event".
- `Aabb::from_points` filters out non-finite points, takes the first remaining
  one as a zero-size box and folds the rest in with component-wise min/max. An
  empty or all-NaN slice gives `None` rather than a nonsense box.
- `Aabb::from_corners` accepts corners in any order, which is what a
  click-and-drag rectangle produces.
- `Aabb::expand` grows each side by a margin (hit-test tolerance, stroke
  width). A negative margin shrinks it; `expand_axis` collapses an axis to its
  midpoint instead of letting `min` pass `max`, so the invariant holds.
- `contains` and `intersects` are inclusive: a point on the edge is inside, two
  boxes sharing an edge intersect. `contains` uses `RangeInclusive::contains`,
  which is also `false` for a NaN point.
- `distance_to_segment` is exactly the formula above, with the degenerate case
  checked first (`len_sq <= 0.0`).

The tests in the same file mirror the spec's acceptance criteria. Two of them
are property tests with `proptest`: for random `p`, `a`, `b` in `[-1000, 1000]²`,
the distance to the segment is finite, non-negative and never larger than the
distance to the nearer endpoint (both endpoints belong to the segment, so the
closest point can only be closer).

## Try it

1. Remove the degenerate-segment check in `distance_to_segment` and run
   `cargo test segment_distance`. Which test fails, and what value does it get?
   (Hint: `NaN.clamp(0.0, 1.0)` is `NaN`.) Restore it.
2. Change `contains` to use exclusive bounds (`<` instead of `..=`). Run the
   tests and read the failure message from `aabb_contains_is_inclusive`.
3. Write `assert!(Vec2::new(0.1, 0.2).x + 0.2 == 0.3);` in a test and run
   `cargo clippy --all-targets`. Then rewrite it with `approx_eq`.
4. Add a proptest `aabb_from_points_contains_every_input` that generates a
   `Vec<Vec2>` and checks the box contains each point.

## Further reading

- *Real-Time Collision Detection*, Christer Ericson — §5.1.2 (closest point on
  a segment) and §4.2 (AABBs).
- [What Every Computer Scientist Should Know About Floating-Point Arithmetic](https://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html)
- rust-skills rules `num-float-compare`, `api-operator-overload`, `api-must-use`,
  `test-proptest-properties`, `test-arrange-act-assert`.
