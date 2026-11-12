#!/usr/bin/env bash
# Compile-fail tests of draw-types (T133, no trybuild): every file in
# crates/draw-types/tests/compile_fail/ must fail to compile with the text
# named in its first line (`// expect: <text>`), and every file in
# crates/draw-types/tests/compile_pass/ must compile.
# Usage: scripts/compile_fail.sh
set -euo pipefail
cd "$(dirname "$0")/.."

export CARGO_BUILD_JOBS=2
cargo build -q -p draw-types

deps=target/debug/deps
rlib=$(ls -t "$deps"/libdraw_types-*.rlib | head -n 1)
out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT

compile() {
    rustc --edition 2024 --crate-type lib --emit metadata \
        --out-dir "$out" -L "dependency=$deps" --extern "draw_types=$rlib" "$1"
}

failed=0

for file in crates/draw-types/tests/compile_pass/*.rs; do
    if ! log=$(compile "$file" 2>&1); then
        echo "FAIL (should compile): $file"
        echo "$log" | tail -n 20
        failed=1
    fi
done

for file in crates/draw-types/tests/compile_fail/*.rs; do
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

if ((failed != 0)); then
    exit 1
fi
echo "compile_fail: all cases behave as expected."
