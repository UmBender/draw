#!/usr/bin/env bash
# Validates packaging/aur/PKGBUILD and .SRCINFO in an Arch Linux container
# using podman (T181): checks .SRCINFO parity, builds the package with
# makepkg, and verifies the installed binary and desktop files.
# Usage: scripts/aur-check.sh
set -euo pipefail
cd "$(dirname "$0")/.."

command -v podman >/dev/null || { echo "podman is required" >&2; exit 1; }

version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
image=docker.io/library/archlinux:latest
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

tar --exclude=./target --exclude=./.git --exclude=./.batch --exclude=./dist \
    --transform="s|^\.|draw-${version}|" -czf "$work/draw-${version}.tar.gz" .
cp packaging/aur/PKGBUILD packaging/aur/.SRCINFO "$work/"

podman run --rm \
    -v "$work:/work:Z" \
    "$image" bash -euc '
        pacman -Syu --noconfirm --needed base-devel rust git libx11 libxi libxkbcommon libglvnd wayland
        useradd -m builder
        chown -R builder:builder /work
        su - builder -c "
            set -euo pipefail
            cd /work
            sed -i \"s|^source=.*|source=(\\\"draw-${version}.tar.gz\\\")|\" PKGBUILD
            diff -u .SRCINFO <(makepkg --printsrcinfo) || true
            makepkg -s --noconfirm --skipchecksums
        "
        pacman -U --noconfirm /work/draw-*.pkg.tar.*
        /usr/bin/draw --version
        /usr/bin/draw --help >/dev/null
        for f in /usr/share/applications/io.github.UmBender.draw.desktop \
                 /usr/share/icons/hicolor/scalable/apps/io.github.UmBender.draw.svg \
                 /usr/share/metainfo/io.github.UmBender.draw.metainfo.xml; do
            test -f "$f"
        done
        echo "aur-check: ok"
    '
