#!/usr/bin/env bash
# Runs every quality gate (docs/process/Quality Gates.md). Stops at the first failure.
# Usage: scripts/check.sh [--fuzz]
set -euo pipefail
cd "$(dirname "$0")/.."

fuzz=false
for arg in "$@"; do
    case "$arg" in
        --fuzz) fuzz=true ;;
        *) echo "unknown argument: $arg" >&2; exit 2 ;;
    esac
done

step() { printf '\n==> %s\n' "$*"; }

step "format";  cargo fmt --all -- --check
step "clippy";  cargo clippy --all-targets -- -D warnings
step "tests";   cargo test --all-targets
step "docs";    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

if $fuzz; then
    if [[ -f tests/fuzz.rs ]]; then
        step "fuzz (20000 cases)"
        PROPTEST_CASES=20000 cargo test --release --test fuzz
    else
        step "fuzz skipped: tests/fuzz.rs does not exist yet"
    fi
fi

printf '\nAll checks passed.\n'
