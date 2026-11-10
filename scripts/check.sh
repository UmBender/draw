#!/usr/bin/env bash
# Runs every quality gate (docs/process/Quality Gates.md). Stops at the first failure.
# Every step stays within the limits below and yields to the desktop (ADR-T77-1).
# Usage: scripts/check.sh [--fuzz]
set -euo pipefail
cd "$(dirname "$0")/.."

readonly JOBS=2
readonly TEST_THREADS=3
readonly MEMORY_MAX=1536M
readonly MEMORY_SWAP_MAX=0
readonly CPU_QUOTA=300%
readonly CPU_WEIGHT=1
readonly IO_WEIGHT=1
readonly NICENESS=19
readonly IO_CLASS_IDLE=3

fuzz=false
for arg in "$@"; do
    case "$arg" in
        --fuzz) fuzz=true ;;
        *) echo "unknown argument: $arg" >&2; exit 2 ;;
    esac
done

export CARGO_BUILD_JOBS=$JOBS
export RUST_TEST_THREADS=$TEST_THREADS

limits=(
    -p "MemoryMax=$MEMORY_MAX"
    -p "MemorySwapMax=$MEMORY_SWAP_MAX"
    -p "CPUQuota=$CPU_QUOTA"
    -p "CPUWeight=$CPU_WEIGHT"
    -p "IOWeight=$IO_WEIGHT"
)

priority=(nice -n "$NICENESS")
if command -v ionice >/dev/null; then
    priority+=(ionice -c "$IO_CLASS_IDLE" -t)
fi

capped=true
if ! systemd-run --user --scope -q "${limits[@]}" -- true >/dev/null 2>&1; then
    capped=false
    echo "check.sh: systemd-run --user is not usable, the memory cap is not enforced" >&2
fi

# gate NAME COMMAND...: runs one step within the limits. NAME's first word
# identifies its unit.
gate() {
    local name=$1
    shift
    printf '\n==> %s\n' "$name"

    if ! $capped; then
        "${priority[@]}" "$@"
        return
    fi

    local unit="draw-check-$$-${name%% *}.scope"
    local status=0
    systemd-run --user --scope -q --unit="$unit" "${limits[@]}" -- \
        "${priority[@]}" "$@" || status=$?
    if ((status != 0)); then
        if [[ $(systemctl --user show -P Result "$unit" 2>/dev/null) == oom-kill ]]; then
            echo "check.sh: $name went over the $MEMORY_MAX memory limit" >&2
        fi
        systemctl --user reset-failed "$unit" 2>/dev/null || true
    fi
    return "$status"
}

gate "format" cargo fmt --all -- --check
gate "clippy" cargo clippy --all-targets -- -D warnings
gate "tests" cargo test --all-targets
RUSTDOCFLAGS="-D warnings" gate "docs" cargo doc --no-deps

if $fuzz; then
    if [[ -f tests/fuzz.rs ]]; then
        PROPTEST_CASES=20000 gate "fuzz (20000 cases)" cargo test --release --test fuzz
    else
        printf '\n==> fuzz skipped: tests/fuzz.rs does not exist yet\n'
    fi
fi

printf '\nAll checks passed.\n'
