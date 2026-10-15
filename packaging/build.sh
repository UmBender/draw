#!/usr/bin/env bash
# Runs inside the image of packaging/Containerfile, started by
# scripts/package.sh: builds the dist binary, splits its debug symbols,
# fills the staging tree and builds the tarball, .deb and .rpm from it
# (ADR-T27-1, ADR-T27-2, ADR-T27-3). Everything goes to target/package/.
set -euo pipefail
cd "$(dirname "$0")/.."

app_id=io.github.UmBender.draw
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
out=target/package
stage=$out/stage/usr
# A target dir of its own: host builds link against a newer glibc.
export CARGO_TARGET_DIR=$out/cargo

step() { printf '\n==> %s\n' "$*"; }

step "build (dist profile, glibc $(ldd --version | head -n 1 | grep -o '[0-9.]*$'))"
RUSTFLAGS="-C link-arg=-Wl,--build-id" cargo build --locked --profile dist
binary=$CARGO_TARGET_DIR/dist/draw

step "split debug symbols"
rm -rf "$out/stage" "$out/draw" "$out/draw.debug"
objcopy --only-keep-debug "$binary" "$out/draw.debug"
objcopy --strip-all --add-gnu-debuglink="$out/draw.debug" "$binary" "$out/draw"

step "stage"
install -Dm755 "$out/draw" "$stage/bin/draw"
install -Dm644 "packaging/$app_id.desktop" "$stage/share/applications/$app_id.desktop"
install -Dm644 packaging/icons/draw.svg "$stage/share/icons/hicolor/scalable/apps/$app_id.svg"
for size in 48 128 256; do
    install -Dm644 "packaging/icons/draw-$size.png" \
        "$stage/share/icons/hicolor/${size}x$size/apps/$app_id.png"
done
install -Dm644 "packaging/$app_id.metainfo.xml" "$stage/share/metainfo/$app_id.metainfo.xml"
install -Dm644 -t "$stage/share/licenses/draw" \
    LICENSE assets/fonts/OFL.txt assets/fonts/GUST-FONT-LICENSE.txt
install -Dm644 -t "$stage/share/doc/draw" README.md CHANGELOG.md

step "tarball"
name=draw-$version-x86_64-linux
rm -rf "$out/$name" "$out/$name.tar.gz"
mkdir -p "$out/$name"
cp -a "$out/stage/usr" "$out/$name/usr"
cp packaging/install.sh README.md CHANGELOG.md LICENSE "$out/$name/"
tar -C "$out" --owner=0 --group=0 -czf "$out/$name.tar.gz" "$name"
rm -rf "$out/$name"

step ".deb"
cargo deb --no-build --no-strip --output "$out/draw_${version}-1_amd64.deb"

step ".rpm"
cargo generate-rpm --output "$out/draw-${version}-1.x86_64.rpm"
