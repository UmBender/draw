#!/usr/bin/env bash
# Exports the Flatpak build into a static OSTree repository in dist/flatpak-repo/
# alongside draw.flatpakrepo so it can be self-hosted (e.g. GitHub Pages)
# without waiting for Flathub review (T183).
# Usage: scripts/flatpak-repo.sh
set -euo pipefail
cd "$(dirname "$0")/.."

app_id=io.github.UmBender.draw
out=target/package
repo=dist/flatpak-repo

if [ ! -d "$out/stage/usr" ]; then
    packaging/build.sh
fi

builder=()
if command -v flatpak-builder >/dev/null; then
    builder=(flatpak-builder)
elif flatpak info org.flatpak.Builder >/dev/null 2>&1; then
    builder=(flatpak run org.flatpak.Builder)
else
    echo "flatpak-builder or org.flatpak.Builder is required" >&2
    exit 1
fi

rm -rf "$repo"
mkdir -p "$repo"

"${builder[@]}" --user --install-deps-from=flathub --force-clean \
    --repo="$repo" "$out/flatpak-build" \
    "packaging/flatpak/$app_id.yml"

flatpak build-update-repo "$repo"
cp packaging/flatpak/draw.flatpakrepo "$repo/draw.flatpakrepo"
cp "packaging/icons/$app_id.svg" "$repo/$app_id.svg"

echo "flatpak-repo: exported static OSTree repo to $repo"
