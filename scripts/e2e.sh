#!/usr/bin/env bash
# Runs the end-to-end suite in a private headless sway session (T75,
# ADR-T75-1). Nothing here can reach the desktop: sway runs on a private
# runtime directory and socket, every child gets that socket and no
# DISPLAY, and the script aborts if the socket is not the one it started.
# The X11 run (T129) is the same suite on a private Xvfb: it listens on a
# local socket only (-nolisten tcp), gets its display number from
# -displayfd, and every child gets that DISPLAY, a private XAUTHORITY and no
# WAYLAND_DISPLAY.
# Usage: scripts/e2e.sh [--update] [--backend wayland|x11|all] [case]
#   --update (or DRAW_UPDATE_SNAPSHOTS=1) rewrites the goldens.
#   --backend picks the display server; the default is all (Wayland, then X11).
set -euo pipefail
cd "$(dirname "$0")/.."

backend=all
while (($#)); do
    case $1 in
        --update)
            export DRAW_UPDATE_SNAPSHOTS=1
            shift
            ;;
        --backend)
            (($# >= 2)) || {
                echo "e2e.sh: --backend needs wayland, x11 or all" >&2
                exit 2
            }
            backend=$2
            shift 2
            ;;
        *) break ;;
    esac
done
case $backend in
    wayland | x11 | all) ;;
    *)
        echo "e2e.sh: unknown backend '$backend' (wayland, x11 or all)" >&2
        exit 2
        ;;
esac
# What the scope re-runs: the parsed options, then the case.
set -- --backend "$backend" "$@"

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

need_tools() {
    for tool in "$@"; do
        command -v "$tool" >/dev/null || die "$tool is not installed"
    done
}

run=$(mktemp -d "${TMPDIR:-/tmp}/draw-e2e.XXXXXX")
mkdir -p target/e2e
sway_pid=
xvfb_pid=
stop_servers() {
    local pid
    for pid in "$sway_pid" "$xvfb_pid"; do
        if [[ -n $pid ]]; then
            kill "$pid" 2>/dev/null || true
            wait "$pid" 2>/dev/null || true
        fi
    done
    sway_pid=
    xvfb_pid=
}
cleanup() {
    stop_servers
    rm -rf -- "$run"
}
trap cleanup EXIT

run_cases() {
    export DRAW_E2E_BACKEND=$1
    export DRAW_E2E_CASE=${2:-}
    [[ -n $DRAW_E2E_CASE ]] || unset DRAW_E2E_CASE
    cargo test --release --test e2e -- --ignored --test-threads=1 --nocapture
}

run_wayland() {
    need_tools sway swaymsg wtype grim
    # Never inherited from the desktop.
    unset DISPLAY WAYLAND_DISPLAY WAYLAND_SOCKET SWAYSOCK I3SOCK XAUTHORITY
    export XDG_RUNTIME_DIR=$run
    export WLR_BACKENDS=headless WLR_HEADLESS_OUTPUTS=1 WLR_RENDERER=pixman
    export WLR_LIBINPUT_NO_DEVICES=1 LIBGL_ALWAYS_SOFTWARE=1

    sway -c tests/e2e/sway.conf >target/e2e/sway.log 2>&1 &
    sway_pid=$!

    local ticks=0 socket= ipc= candidate
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

    run_cases wayland "$@"
}

run_x11() {
    need_tools Xvfb xdotool import
    # Never inherited from the desktop.
    unset DISPLAY WAYLAND_DISPLAY WAYLAND_SOCKET SWAYSOCK I3SOCK XAUTHORITY
    export XDG_RUNTIME_DIR=$run LIBGL_ALWAYS_SOFTWARE=1
    export XAUTHORITY=$run/Xauthority
    : >"$XAUTHORITY"

    # The server writes its display number to fd 3, which is this file.
    local shown=$run/display
    Xvfb -nolisten tcp -displayfd 3 -screen 0 1280x1024x24 -noreset \
        3>"$shown" >target/e2e/xvfb.log 2>&1 &
    xvfb_pid=$!

    local ticks=0 number=
    while [[ -z $number ]]; do
        kill -0 "$xvfb_pid" 2>/dev/null || die "Xvfb exited, see target/e2e/xvfb.log"
        ((ticks++ < START_TIMEOUT_TICKS)) || die "Xvfb did not start, see target/e2e/xvfb.log"
        sleep "$TICK"
        number=$(tr -d '[:space:]' <"$shown")
    done

    export DISPLAY=:$number DRAW_E2E_RUNTIME=$run

    # The last check before any input tool can run.
    [[ $number =~ ^[0-9]+$ ]] || die "refusing to run: DISPLAY=$DISPLAY is not a local display"
    [[ $XAUTHORITY == "$run"/* && -f $XAUTHORITY ]] ||
        die "refusing to run: XAUTHORITY=$XAUTHORITY is not the private file"
    [[ -z ${WAYLAND_DISPLAY:-} ]] || die "refusing to run: WAYLAND_DISPLAY is set"

    run_cases x11 "$@"
    stop_servers
}

case $backend in
    wayland) run_wayland "$@" ;;
    x11) run_x11 "$@" ;;
    all)
        # Both must pass; the Wayland failure does not hide the X11 result.
        status=0
        (
            trap stop_servers EXIT
            run_wayland "$@"
        ) || status=$?
        (
            trap stop_servers EXIT
            run_x11 "$@"
        ) || status=$?
        exit "$status"
        ;;
esac
