#!/usr/bin/env bash
# Lines per source file and per top-level directory (T127 baseline).
# Usage: scripts/audit/sizes.sh
set -euo pipefail
cd "$(dirname "$0")/../.."

echo "## Lines per directory"
echo
echo "| Directory | Files | Lines |"
echo "|-----------|-------|-------|"
for d in src/core src/shell src vendor/miniquad tests; do
  [ -d "$d" ] || continue
  files=$(find "$d" -name '*.rs' | wc -l)
  lines=$(find "$d" -name '*.rs' -print0 | xargs -0 cat | wc -l)
  echo "| \`$d\` | $files | $lines |"
done
echo
echo "## Largest files in src"
echo
echo "| File | Lines |"
echo "|------|-------|"
find src -name '*.rs' -print0 | xargs -0 wc -l | grep -v ' total$' \
  | sort -rn | head -12 | awk '{printf "| `%s` | %s |\n", $2, $1}'
