#!/usr/bin/env bash
# Runs the integration tests of draw-sys under the address and thread
# sanitizers (AC-3 of T142). Needs a nightly toolchain with rust-src.
# Usage: scripts/unsafe-sanitizers.sh [address|thread]...   (default: both)
set -euo pipefail
cd "$(dirname "$0")/.."

export CARGO_BUILD_JOBS=2
target=$(rustc +nightly -vV | sed -n 's/^host: //p')

sanitizers=("$@")
if ((${#sanitizers[@]} == 0)); then
    sanitizers=(address thread)
fi

for san in "${sanitizers[@]}"; do
    case "$san" in
        address | thread) ;;
        *) echo "unknown sanitizer: $san" >&2; exit 2 ;;
    esac
    printf '\n==> %s sanitizer\n' "$san"
    # The sanitized std must match the instrumented code, hence build-std.
    # A separate target dir keeps the flags from rebuilding the normal one.
    RUSTFLAGS="-Zsanitizer=$san" RUSTDOCFLAGS="-Zsanitizer=$san" \
        CARGO_TARGET_DIR="target/sanitizer-$san" \
        cargo +nightly test -Zbuild-std --target "$target" \
        -p draw-sys --test loader --test unsafe_refs
done
