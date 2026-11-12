#!/usr/bin/env bash
# Line coverage per module with cargo-llvm-cov, checked against the ratchet
# in coverage-baseline.toml (T130, docs/process/Quality Gates.md).
#
# A module is a top-level directory of src/ (core, shell); files directly
# in src/ and outside src/ are not counted. A module may fall
# 0.5 points below its recorded value, and may rise 0.5 points above it;
# beyond that the build fails: a fall is a regression, a rise must be
# recorded by updating coverage-baseline.toml (`scripts/coverage.sh --update`).
#
# Usage: scripts/coverage.sh [--summary FILE] [--baseline FILE] [--update]
#   --summary FILE   use an existing `cargo llvm-cov --json` summary
#   --baseline FILE  baseline file (default: coverage-baseline.toml)
#   --update         rewrite the baseline with the measured values
#
# The run uses the same resource caps as scripts/check.sh (ADR-T77-1).
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
readonly TOLERANCE=5 # tenths of a point

summary=""
baseline="coverage-baseline.toml"
update=false
while (($#)); do
    case "$1" in
        --summary) summary=${2:?--summary needs a file}; shift 2 ;;
        --baseline) baseline=${2:?--baseline needs a file}; shift 2 ;;
        --update) update=true; shift ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done

command -v jq >/dev/null || { echo "coverage.sh: jq is required" >&2; exit 2; }

export CARGO_BUILD_JOBS=$JOBS
export RUST_TEST_THREADS=$TEST_THREADS

# Runs the tests under coverage within the limits.
measure() {
    local out=$1
    local limits=(
        -p "MemoryMax=$MEMORY_MAX"
        -p "MemorySwapMax=$MEMORY_SWAP_MAX"
        -p "CPUQuota=$CPU_QUOTA"
        -p "CPUWeight=$CPU_WEIGHT"
        -p "IOWeight=$IO_WEIGHT"
    )
    local priority=(nice -n "$NICENESS")
    if command -v ionice >/dev/null; then
        priority+=(ionice -c "$IO_CLASS_IDLE" -t)
    fi
    local cmd=(cargo llvm-cov --summary-only --json --output-path "$out")
    mkdir -p "$(dirname "$out")"
    if systemd-run --user --scope -q "${limits[@]}" -- true >/dev/null 2>&1; then
        systemd-run --user --scope -q "${limits[@]}" -- "${priority[@]}" "${cmd[@]}" >&2
    else
        echo "coverage.sh: systemd-run --user is not usable, the memory cap is not enforced" >&2
        "${priority[@]}" "${cmd[@]}" >&2
    fi
}

if [[ -z $summary ]]; then
    summary=target/llvm-cov-summary.json
    measure "$summary"
fi

# Tenths of a point per module: "<module> <tenths>".
declare -A current=()
while read -r module lines covered; do
    if ((lines == 0)); then tenths=0; else tenths=$(((covered * 2000 / lines + 1) / 2)); fi
    current[$module]=$tenths
done < <(jq -r '
    .data[0].files
    | map(select(.filename | test("/src/[^/]+/"))
          | {m: ([.filename | match("/src/([^/]+)/"; "g").captures[0].string] | last),
             c: .summary.lines.count, v: .summary.lines.covered})
    | group_by(.m)[]
    | "\(.[0].m) \(map(.c) | add) \(map(.v) | add)"' "$summary")

fmt() { printf '%d.%d' $(($1 / 10)) $(($1 % 10)); }

modules=("${!current[@]}")
mapfile -t modules < <(printf '%s\n' "${modules[@]}" | sort)
for module in "${modules[@]}"; do
    echo "$module $(fmt "${current[$module]}")"
done

if $update; then
    {
        echo "# Line coverage per module in percent (scripts/coverage.sh, T130)."
        echo "# CI fails when a module falls or rises by more than 0.5 points."
        for module in "${modules[@]}"; do
            echo "$module = $(fmt "${current[$module]}")"
        done
    } >"$baseline"
    echo "coverage.sh: wrote $baseline"
    exit 0
fi

declare -A recorded=()
while IFS='=' read -r key value; do
    key=${key//[[:space:]]/}
    value=${value//[[:space:]]/}
    [[ -z $key || $key == \#* ]] && continue
    whole=${value%%.*}
    frac=${value#*.}
    [[ $value == *.* ]] || frac=0
    recorded[$key]=$((10#$whole * 10 + 10#${frac:0:1}))
done <"$baseline"

failed=false
for module in "${modules[@]}"; do
    now=${current[$module]}
    if [[ -z ${recorded[$module]+x} ]]; then
        echo "coverage.sh: $module: no baseline in $baseline (run scripts/coverage.sh --update)" >&2
        failed=true
        continue
    fi
    was=${recorded[$module]}
    if ((now < was - TOLERANCE)); then
        echo "coverage.sh: $module fell from $(fmt "$was") to $(fmt "$now") (more than 0.5 points): add tests" >&2
        failed=true
    elif ((now > was + TOLERANCE)); then
        echo "coverage.sh: $module rose from $(fmt "$was") to $(fmt "$now"): raise it in coverage-baseline.toml (scripts/coverage.sh --update)" >&2
        failed=true
    fi
done
for module in "${!recorded[@]}"; do
    if [[ -z ${current[$module]+x} ]]; then
        echo "coverage.sh: $module is in $baseline but was not measured" >&2
        failed=true
    fi
done

if $failed; then exit 1; fi
echo "coverage.sh: all modules within the baseline"
