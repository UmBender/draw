#!/usr/bin/env bash
# Runs the end-to-end suite in a private headless sway session (T75,
# ADR-T75-1). Nothing here can reach the desktop: sway runs on a private
# runtime directory and socket, every child gets that socket and no
# DISPLAY, and the script aborts if the socket is not the one it started.
# Usage: scripts/e2e.sh [--update] [case]
#   --update (or DRAW_UPDATE_SNAPSHOTS=1) rewrites the goldens.
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ ${1:-} == --update ]]; then
    export DRAW_UPDATE_SNAPSHOTS=1
    shift
fi

readonly JOBS=3
readonly MEMORY_MAX=1G
readonly MEMORY_SWAP_MAX=0
readonly CPU_QUOTA=300%
readonly CPU_WEIGHT=1
readonly IO_WEIGHT=1
readonly NICENESS=19
readonly IO_CLASS_IDLE=3
readonly START_TIMEOUT_TICKS=600
readonly TICK=0.05

# The scope starts first: systemd-run --user needs the real runtime directory.
if [[ ${DRAW_E2E_IN_SCOPE:-} != 1 ]]; then
    export DRAW_E2E_IN_SCOPE=1 CARGO_BUILD_JOBS=$JOBS
    priority=(nice -n "$NICENESS")
    if command -v ionice >/dev/null; then
        priority+=(ionice -c "$IO_CLASS_IDLE" -t)
    fi
    limits=(
        -p "MemoryMax=$MEMORY_MAX"
        -p "MemorySwapMax=$MEMORY_SWAP_MAX"
        -p "CPUQuota=$CPU_QUOTA"
        -p "CPUWeight=$CPU_WEIGHT"
        -p "IOWeight=$IO_WEIGHT"
    )
    if systemd-run --user --scope -q "${limits[@]}" -- true >/dev/null 2>&1; then
        unit="draw-e2e-$$.scope"
        status=0
        systemd-run --user --scope -q --unit="$unit" "${limits[@]}" -- \
            "${priority[@]}" "$0" "$@" || status=$?
        if ((status != 0)); then
            if [[ $(systemctl --user show -P Result "$unit" 2>/dev/null) == oom-kill ]]; then
                echo "e2e.sh: went over the $MEMORY_MAX memory limit" >&2
            fi
            systemctl --user reset-failed "$unit" 2>/dev/null || true
        fi
        exit "$status"
    fi
    echo "e2e.sh: systemd-run --user is not usable, the memory cap is not enforced" >&2
    exec "${priority[@]}" "$0" "$@"
fi

die() {
    echo "e2e.sh: $*" >&2
    exit 1
}

for tool in sway swaymsg wtype grim; do
    command -v "$tool" >/dev/null || die "$tool is not installed"
done

run=$(mktemp -d "${TMPDIR:-/tmp}/draw-e2e.XXXXXX")
mkdir -p target/e2e
sway_pid=
cleanup() {
    if [[ -n $sway_pid ]]; then
        kill "$sway_pid" 2>/dev/null || true
        wait "$sway_pid" 2>/dev/null || true
    fi
    rm -rf -- "$run"
}
trap cleanup EXIT

# Never inherited from the desktop.
unset DISPLAY WAYLAND_DISPLAY WAYLAND_SOCKET SWAYSOCK I3SOCK
export XDG_RUNTIME_DIR=$run
export WLR_BACKENDS=headless WLR_HEADLESS_OUTPUTS=1 WLR_RENDERER=pixman
export WLR_LIBINPUT_NO_DEVICES=1 LIBGL_ALWAYS_SOFTWARE=1

sway -c tests/e2e/sway.conf >target/e2e/sway.log 2>&1 &
sway_pid=$!

ticks=0
socket=
ipc=
while [[ -z $socket || -z $ipc ]]; do
    kill -0 "$sway_pid" 2>/dev/null || die "sway exited, see target/e2e/sway.log"
    ((ticks++ < START_TIMEOUT_TICKS)) || die "sway did not start, see target/e2e/sway.log"
    sleep "$TICK"
    for candidate in "$run"/wayland-*; do
        [[ -S $candidate ]] && socket=$candidate
    done
    for candidate in "$run"/sway-ipc.*.sock; do
        [[ -S $candidate ]] && ipc=$candidate
    done
done

export WAYLAND_DISPLAY=$socket SWAYSOCK=$ipc DRAW_E2E_RUNTIME=$run

# The last check before any input tool can run.
[[ $WAYLAND_DISPLAY == "$run"/wayland-* && -S $WAYLAND_DISPLAY ]] ||
    die "refusing to run: $WAYLAND_DISPLAY is not the private socket"
[[ $SWAYSOCK == "$run"/* && -S $SWAYSOCK ]] ||
    die "refusing to run: $SWAYSOCK is not the private socket"

export DRAW_E2E_CASE=${1:-}
[[ -n $DRAW_E2E_CASE ]] || unset DRAW_E2E_CASE
cargo test --release --test e2e -- --ignored --test-threads=1 --nocapture
