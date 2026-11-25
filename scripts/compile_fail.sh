#!/usr/bin/env bash
# Compile-fail tests of draw-types (T133) and draw-sys (T168), no trybuild:
# every file in crates/<crate>/tests/compile_fail/ must fail to compile with
# the text named in its first line (`// expect: <text>`), and every file in
# crates/<crate>/tests/compile_pass/ must compile.
# Usage: scripts/compile_fail.sh
set -euo pipefail
cd "$(dirname "$0")/.."

export CARGO_BUILD_JOBS=2
out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT

failed=0

# check_crate <package> <crate name with underscores> <directory>
#
# The rlibs come from Cargo's own artifact messages, never from a guessed
# directory: Cargo 1.100 moved the intermediate files out of
# target/debug/deps (the build-dir layout), which broke `ls deps/lib*`
# (T194).
check_crate() {
    local package=$1 crate=$2 dir=$3 rlib file expect log path
    local -a rlibs search=()
    mapfile -t rlibs < <(cargo build -q -p "$package" --message-format=json |
        grep -o '"[^"]*\.rlib"' | tr -d '"' | sort -u)
    rlib=$(printf '%s\n' "${rlibs[@]}" | grep -E "/lib${crate}(-[0-9a-f]+)?\.rlib$" | head -n 1)
    if [[ -z $rlib ]]; then
        echo "compile_fail: cargo reported no rlib for $package" >&2
        exit 2
    fi
    for path in "${rlibs[@]}"; do
        search+=(-L "dependency=$(dirname "$path")")
    done

    compile() {
        rustc --edition 2024 --crate-type lib --emit metadata \
            --out-dir "$out" "${search[@]}" --extern "$crate=$rlib" "$1"
    }

    for file in "$dir"/tests/compile_pass/*.rs; do
        [[ -e $file ]] || continue
        if ! log=$(compile "$file" 2>&1); then
            echo "FAIL (should compile): $file"
            echo "$log" | tail -n 20
            failed=1
        fi
    done

    for file in "$dir"/tests/compile_fail/*.rs; do
        [[ -e $file ]] || continue
        expect=$(sed -n '1s|^// expect: ||p' "$file")
        if [[ -z $expect ]]; then
            echo "FAIL (no '// expect:' first line): $file"
            failed=1
            continue
        fi
        if log=$(compile "$file" 2>&1); then
            echo "FAIL (compiled, should not): $file"
            failed=1
        elif ! grep -qF -- "$expect" <<<"$log"; then
            echo "FAIL (wrong error, wanted '$expect'): $file"
            echo "$log" | tail -n 20
            failed=1
        fi
    done
}

check_crate draw-types draw_types crates/draw-types
check_crate draw-sys draw_sys crates/draw-sys

if ((failed != 0)); then
    exit 1
fi
echo "compile_fail: all cases behave as expected."
