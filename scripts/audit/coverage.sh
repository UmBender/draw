#!/usr/bin/env bash
# Line coverage per source file with cargo-llvm-cov, then per module
# (T127 baseline). Slow: run by hand, once.
# Usage: scripts/audit/coverage.sh [out.json]
set -euo pipefail
cd "$(dirname "$0")/../.."

out="${1:-target/llvm-cov-summary.json}"
cargo llvm-cov --summary-only --json --output-path "$out" >/dev/null

# Summarise by directory of src/ (core, core/tools, shell, ...). jq is used
# when present; otherwise the raw JSON path is printed.
if ! command -v jq >/dev/null; then
  echo "jq missing: raw summary in $out"
  exit 0
fi
echo "| Module | Lines | Covered | % |"
echo "|--------|-------|---------|---|"
jq -r --arg root "$PWD/" '
  .data[0].files
  | map({m: (.filename | ltrimstr($root) | split("/") | .[0:2] | join("/")),
         c: .summary.lines.count, v: .summary.lines.covered})
  | group_by(.m)[]
  | {m: .[0].m, c: (map(.c) | add), v: (map(.v) | add)}
  | "| `\(.m)` | \(.c) | \(.v) | \(if .c == 0 then 0 else (.v * 1000 / .c | round / 10) end) |"' "$out"
