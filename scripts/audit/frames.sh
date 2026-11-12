#!/usr/bin/env bash
# Frame-time baseline: prints how to run the ADR-T24-5 scene and, when a
# display and a saved scene are given, runs the app with DRAW_FRAME_TIMES=1.
# Needs a real window; on the owner's i5-8250U this fills the report table.
# Usage: scripts/audit/frames.sh [scene.draw]
set -euo pipefail
cd "$(dirname "$0")/../.."

if [ -z "${WAYLAND_DISPLAY:-}${DISPLAY:-}" ]; then
  echo "no display: not measured on the i5-8250U"
  exit 0
fi
cargo build --release >/dev/null
echo "Draw on the scene, pan, zoom and quit; frame times follow (stderr)."
DRAW_FRAME_TIMES=1 target/release/draw ${1:+"$1"} 2>&1 >/dev/null | tail -n 40
