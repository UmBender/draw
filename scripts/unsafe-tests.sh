#!/usr/bin/env bash
# Checks that every `unsafe` block names its test with `// TEST: <fn_name>`
# (docs/process/Unsafe primitives.md, ADR-T142-1), then runs Miri on the
# unit tests of draw-sys when a nightly toolchain with Miri is installed.
# Usage: scripts/unsafe-tests.sh
set -euo pipefail
cd "$(dirname "$0")/.."

export CARGO_BUILD_JOBS=2

cargo test -q -p draw-sys --test unsafe_refs

if cargo +nightly miri --version >/dev/null 2>&1; then
    cargo +nightly miri test -q -p draw-sys --lib
else
    echo "unsafe-tests: nightly Miri is not installed, skipped (rustup +nightly component add miri)" >&2
fi
