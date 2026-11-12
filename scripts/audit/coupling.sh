#!/usr/bin/env bash
# Who imports whom: `use crate::<module>` edges between top-level modules of
# src/core and src/shell, plus the files that use macroquad (T127 baseline).
# Usage: scripts/audit/coupling.sh
set -eu # no pipefail: grep exits 1 on "no match", which is a valid count of 0
cd "$(dirname "$0")/../.."

echo "## Imports between layers"
echo
echo "| From | To | Files with the import |"
echo "|------|----|-----------------------|"
core_to_shell=$(grep -rlE 'crate::shell|super::shell' src/core 2>/dev/null | wc -l)
shell_to_core=$(grep -rlE 'crate::core' src/shell 2>/dev/null | wc -l)
echo "| core | shell | $core_to_shell |"
echo "| shell | core | $shell_to_core |"
echo
echo "## Files using macroquad"
echo
mq_files=$(grep -rl 'macroquad' src | sort)
echo "Total: $(echo "$mq_files" | grep -c .)"
echo
echo "$mq_files" | sed 's/^/- `/; s/$/`/'
echo
echo "## core modules by number of importing files (inside core)"
echo
echo "| Module | Imported by (files) |"
echo "|--------|---------------------|"
for f in src/core/*.rs; do
  m=$(basename "$f" .rs)
  [ "$m" = mod ] && continue
  n=$(grep -rlE "(crate::core|super)::$m\b|use super::\{?[^;]*\b$m\b" src/core --include='*.rs' \
      | grep -v "src/core/$m.rs" | wc -l)
  echo "| $m | $n |"
done | sort -t'|' -k3 -rn
