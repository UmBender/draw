---
id: ADR-T104-1
title: Segment tree keyword
status: accepted
kind: decision
date: 2026-11-10
task: T104
builds_on: [ADR-T87-1, ADR-T96-1, ADR-T98-1, ADR-T86-1]
amends: [ADR-T96-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T104-1 Segment tree keyword

## Context

Drawing a segment tree by hand means writing `2n − 1` circles and their ranges. [[ADR-T96-1 Keywords for graph input]] lets a first line say how a block is read, and [[ADR-T98-1 Math names and weights in graphs]] already draws `$…$` names inside circles sized to fit. Positions start at 1, or at 0 after `Ctrl+N` ([[ADR-T86-1 Zero-based numbering switch]]), but `EdgeList::parse` does not know the editor's first number.

## Decision

- **Keyword.** A first line that, without outer whitespace, starts with `segtree:` asks for a segment tree. Unlike the other keywords, the size may follow on the same line: `segtree: 8` and `segtree:` then `8` are the same. `segtree:8` is read too.
- **Size.** Every whitespace-separated token after the colon, on that line and the lines after it, is an entry. Exactly one entry, a number `n` with `1 ≤ n ≤ 100`, is the size. A second entry, a number out of range, or a token that is not a number refuses the text with the new `GraphError::NotATreeSize`, which names the line of the bad entry (the second entry, when there are two). No entry at all is no graph, like any keyword alone.
- **First number.** `EdgeList::parse_numbered(text, first)` reads the text with positions starting at `first`; `EdgeList::parse(text)` is `parse_numbered(text, 1)`. The editor passes its current first number (1, or 0 after `Ctrl+N`). Only `segtree:` uses it; every other format reads as before.
- **Tree.** The root is `[f, f+n−1]`. A node `[l, r]` with `l < r` has the children `[l, m]` and `[m+1, r]`, `m = ⌊(l+r)/2⌋`; `[i, i]` is a leaf. Nodes are named `$[l, r]$` (`NodeName::Math`) in preorder, left child before right, and each child's edge is undirected and written right after its parent is named. The ordinary layered tree layout of [[ADR-T87-1 Graph from an edge list key and layout rules]] then draws it from the root, children left to right, and every circle takes the size of the widest label.
- **Limit.** `n ≤ 100` gives at most 199 nodes, under `MAX_NODES`.

## Alternatives considered

- **A separate parser and drawing** — duplicates the layered layout and the circle sizing that `EdgeList` already has.
- **Reusing `GraphError::NotAnEdge`** — its message "line N is not an edge" is wrong for a size.
- **Refusing `segtree:` with no size** — every other keyword alone is no graph; kept the same.
- **Half-open ranges or values in nodes** — out of scope for T104.

## Consequences

- `Ctrl+L` on a block now depends on the zero-based switch, for `segtree:` only.
- A future `fenwick:` or similar template can follow the same pattern: build `RawEdge`s with math names and let the layout do the rest.

## Rollback plan

Revert the T104 commits and record it in a rollback ADR. Trees already drawn stay as they are.
