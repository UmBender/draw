#!/usr/bin/env bash
# Counts of `dyn`, `Box<dyn` and `unsafe` in src and vendor/miniquad (T127).
# Lines are counted, not occurrences; comment-only lines are skipped.
# Usage: scripts/audit/unsafe-dyn.sh
set -euo pipefail
cd "$(dirname "$0")/../.."

count() { # pattern dir
  grep -rE --include='*.rs' "$1" "$2" | grep -vE '^[^:]*:[[:space:]]*//' | wc -l
}

echo "| Pattern | src | vendor/miniquad |"
echo "|---------|-----|-----------------|"
echo "| \`dyn \` | $(count '\bdyn ' src) | $(count '\bdyn ' vendor/miniquad) |"
echo "| \`Box<dyn\` | $(count 'Box<dyn' src) | $(count 'Box<dyn' vendor/miniquad) |"
echo "| \`unsafe\` | $(count '\bunsafe\b' src) | $(count '\bunsafe\b' vendor/miniquad) |"
