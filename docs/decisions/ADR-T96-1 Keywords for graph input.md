---
id: ADR-T96-1
title: Keywords for graph input
status: accepted
kind: decision
date: 2026-11-09
task: T96
builds_on: [ADR-T87-1]
amends: [ADR-T87-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T96-1 Keywords for graph input

## Context

[[ADR-T87-1 Graph from an edge list key and layout rules]] reads plain `u v` lines as undirected edges. A problem's input often gives directed edges as `u v`, gives a tree as a parent list, and starts with a count line. Turning it into T87's grammar means editing every line.

## Decision

- **Keyword line.** The first non-blank line of the block, without outer whitespace, may be exactly one of `graph:`, `tree:`, `dgraph:`, `parents:` or `parents0:`. It says how the lines after it are read. Without one, T87's grammar applies unchanged. Any other first line is read as an edge, so a misspelt keyword is a bad line.
- **`graph:` and `tree:`** read T87 edges. `tree:` does not force a tree: the shape of the graph still decides the layout.
- **`dgraph:`** reads `u v` and `u v w` as directed from `u` to `v`. Written pairs keep their meaning: `u-v` is undirected, `u>v` directed.
- **Count line for edges.** After an edge keyword, the first line is skipped when it is one number (`n`), or two numbers `n m` with `m` greater than 0 and equal to the number of non-blank lines after it, and `n` at least every node name in those lines. A first edge `u v` is read as a count line only when `v` equals the number of lines after it and `u` is at least every node name in them; this is the price of pasting input unchanged.
- **Parent lists.** After `parents:` or `parents0:`, every whitespace-separated token on every line is an entry; line breaks do not matter.
  - Without `-1`, `parents:` lists the parents of nodes 2, 3, … with root 1, and `parents0:` those of nodes 1, 2, … with root 0.
  - With exactly one `-1`, the list has one entry per node, numbered from 1 (`parents:`) or 0 (`parents0:`), and the `-1` entry is the root.
  - A first line holding one number `n` is a count line, skipped when `n − 1` or `n` entries follow it.
  - Every other entry is a node name as in T87. Another `-1`, any other negative or a non-number refuses the text and names the line.
  - Each entry gives an undirected edge from the parent to the node, in list order. The root is named first, so a tree is drawn from it.
- **Blank.** A keyword with no entries after it is no graph, like blank text.

## Alternatives considered

- **Keywords without a colon** — `tree` alone looks like a typo more than a command, and could one day be a node name.
- **Always skip the line after a keyword** — loses the first edge of input that has no count line.
- **A root marked with `0`** — collides with node 0 in `parents:` lists; left out of scope.

## Consequences

Input can be pasted with one line added on top. The count line heuristic can swallow one real edge in the rare case above. Keyword lists go through the same node limit and refusals as T87.

## Rollback plan

Revert the T96 commits and record it in a rollback ADR. Graphs already drawn stay as they are.
