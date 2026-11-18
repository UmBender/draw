#!/usr/bin/env bash
# Builds and checks the Fedora COPR RPM spec (packaging/rpm/draw.spec) in a
# Fedora container using podman (T182), using a vendored offline build.
# Usage: scripts/copr-check.sh
set -euo pipefail
cd "$(dirname "$0")/.."

command -v podman >/dev/null || { echo "podman is required" >&2; exit 1; }
command -v cargo >/dev/null || { echo "cargo is required" >&2; exit 1; }

version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
image=registry.fedoraproject.org/fedora:latest
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

tar --exclude=./target --exclude=./.git --exclude=./.batch --exclude=./dist \
    --transform="s|^\.|draw-${version}|" -czf "$work/draw-${version}.tar.gz" .
cargo vendor --locked "$work/vendor" >/dev/null
tar -C "$work" -czf "$work/draw-${version}-vendor.tar.gz" vendor
rm -rf "$work/vendor"
cp packaging/rpm/draw.spec "$work/"

podman run --rm \
    -v "$work:/work:Z" \
    "$image" bash -euc '
        dnf install -y rpm-build cargo rust libX11-devel libXi-devel libxkbcommon-devel libglvnd-devel wayland-devel
        mkdir -p ~/rpmbuild/{SOURCES,SPECS}
        cp /work/draw-*.tar.gz ~/rpmbuild/SOURCES/
        cp /work/draw.spec ~/rpmbuild/SPECS/
        rpmbuild -ba ~/rpmbuild/SPECS/draw.spec
        dnf install -y ~/rpmbuild/RPMS/*/draw-*.rpm
        /usr/bin/draw --version
        /usr/bin/draw --help >/dev/null
        for f in /usr/share/applications/io.github.UmBender.draw.desktop \
                 /usr/share/icons/hicolor/scalable/apps/io.github.UmBender.draw.svg \
                 /usr/share/metainfo/io.github.UmBender.draw.metainfo.xml; do
            test -f "$f"
        done
        echo "copr-check: ok"
    '
