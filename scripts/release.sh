#!/usr/bin/env bash
# The release path (T129): check.sh --fuzz, then the end-to-end suite on
# Wayland and on X11, then package.sh. A failing step stops the release
# before anything is packaged.
# Usage: scripts/release.sh [--dry-run] X.Y.Z
#   X.Y.Z must be the version in Cargo.toml.
#   --dry-run prints the steps without running them; with
#   DRAW_RELEASE_FAIL=<step> that step fails, to test the stop.
set -euo pipefail
cd "$(dirname "$0")/.."

dry=0
if [[ ${1:-} == --dry-run ]]; then
    dry=1
    shift
fi

usage() {
    echo "release.sh: $*" >&2
    echo "usage: scripts/release.sh [--dry-run] X.Y.Z" >&2
    exit 2
}

version=${1:-}
[[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || usage "the version must look like X.Y.Z"
manifest=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
[[ $manifest == "$version" ]] ||
    usage "$version is not the version in Cargo.toml ($manifest)"

# step <name> <command...>: runs it, or only prints it in a dry run.
step() {
    local name=$1
    shift
    printf '\n==> %s\n' "$name"
    if ((dry)); then
        echo "dry-run: $*"
        if [[ ${DRAW_RELEASE_FAIL:-} == "$name" ]]; then
            echo "release.sh: step $name failed" >&2
            exit 1
        fi
    elif ! "$@"; then
        echo "release.sh: step $name failed" >&2
        exit 1
    fi
}

step check scripts/check.sh --fuzz
step e2e-wayland scripts/e2e.sh --backend wayland
step e2e-x11 scripts/e2e.sh --backend x11
step package env DRAW_RELEASE=1 scripts/package.sh
printf '\nrelease.sh: %s is ready in target/package\n' "$version"
