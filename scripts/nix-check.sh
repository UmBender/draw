#!/usr/bin/env bash
# Builds and checks the Nix flake in a nixos/nix container (T174); Nix is
# not needed on the host, only podman and a network (cache.nixos.org,
# github.com).
# Usage: scripts/nix-check.sh [--lock]
#   --lock   also write the flake.lock the container resolved back here
set -euo pipefail
cd "$(dirname "$0")/.."

lock=false
[ "${1:-}" = "--lock" ] && lock=true

image=docker.io/nixos/nix:latest
out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT

podman run --rm \
    -v "$PWD":/src:ro,Z \
    -v "$out":/out:Z \
    "$image" sh -euc '
        nix="nix --extra-experimental-features nix-command --extra-experimental-features flakes"
        mkdir /work
        cd /src
        tar --exclude=./target --exclude=./.git --exclude=./.batch --exclude=./dist -cf - . | tar -xf - -C /work
        cd /work
        $nix flake lock
        cp flake.lock /out/flake.lock
        $nix flake check --print-build-logs
        $nix build --print-build-logs
        ./result/bin/draw --version
        ./result/bin/draw --help > /dev/null
        # The wrapper must carry every library of the flake.
        grep -c LD_LIBRARY_PATH result/bin/draw
        for f in share/applications/io.github.UmBender.draw.desktop \
                 share/icons/hicolor/scalable/apps/io.github.UmBender.draw.svg \
                 share/metainfo/io.github.UmBender.draw.metainfo.xml; do
            test -f "result/$f"
        done
        # Every soname dlopen needs resolves through the wrapper path.
        path=$(grep -o "/nix/store/[^: \"]*/lib" result/bin/draw | tr "\n" ":")
        for so in libX11.so.6 libXi.so.6 libxkbcommon.so.0 libGL.so.1 libEGL.so.1 \
                  libwayland-client.so.0 libwayland-egl.so.1 libdecor-0.so.0; do
            found=false
            IFS=:
            for dir in $path; do [ -e "$dir/$so" ] && found=true; done
            unset IFS
            $found || { echo "missing $so in the wrapper path" >&2; exit 1; }
        done
        echo "nix-check: ok"
    '

if $lock; then
    cp "$out/flake.lock" flake.lock
    echo "flake.lock written"
fi
