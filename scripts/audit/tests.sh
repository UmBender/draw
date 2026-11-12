#!/usr/bin/env bash
# Test functions per source file group (`#[test]` and proptest! cases) and in
# tests/ (T127 baseline). Counts attributes, not runs.
# Usage: scripts/audit/tests.sh
set -euo pipefail
cd "$(dirname "$0")/../.."

echo "| Location | #[test] | proptest cases |"
echo "|----------|---------|----------------|"
for d in src/core src/core/tools src/shell tests; do
  [ -d "$d" ] || continue
  t=$(find "$d" -maxdepth 1 -name '*.rs' -print0 | xargs -0 cat 2>/dev/null | grep -c '#\[test\]' || true)
  p=$(find "$d" -maxdepth 1 -name '*.rs' -print0 | xargs -0 cat 2>/dev/null | grep -cE '^\s*proptest!' || true)
  echo "| \`$d\` | $t | $p |"
done
echo
echo "## Per file in src/core (top 12 by #[test])"
echo
echo "| File | #[test] |"
echo "|------|---------|"
for f in src/core/*.rs src/core/tools/*.rs; do
  n=$(grep -c '#\[test\]' "$f" || true)
  echo "$n $f"
done | sort -rn | head -12 | awk '{printf "| `%s` | %s |\n", $2, $1}'
