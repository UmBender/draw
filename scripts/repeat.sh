#!/usr/bin/env bash
# Runs a command N times and stops at the first failure.
# Usage: scripts/repeat.sh <n> <cmd> [args...]
set -u
n="$1"
shift
for i in $(seq 1 "$n"); do
    if ! "$@" >/dev/null 2>&1; then
        echo "FAILED on run $i of $n" >&2
        exit 1
    fi
done
echo "OK: $n consecutive green runs"
