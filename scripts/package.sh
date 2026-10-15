#!/usr/bin/env bash
# Builds the Linux release packages into dist/ (T27): a portable tarball, a
# .deb, an .rpm, a local Flatpak bundle when flatpak-builder is available,
# the split debug symbols and SHA256SUMS. Needs only podman (plus
# flatpak-builder for the bundle); fails if any packaging check fails.
# Usage: scripts/package.sh [--no-flatpak]
#        scripts/package.sh --check   (inside the build container)
set -euo pipefail
cd "$(dirname "$0")/.."

app_id=io.github.UmBender.draw
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
out=target/package
tarball=$out/draw-$version-x86_64-linux.tar.gz
glibc_floor=2.28
max_binary_bytes=$((2 * 1024 * 1024))

step() { printf '\n==> %s\n' "$*"; }
fail() { echo "package.sh: $*" >&2; exit 1; }

# AC-1: every lib*.so the shipped binary names is in packaging/runtime-libs.
check_runtime_libs() {
    local listed mentioned base missing=()
    listed=$(grep -v '^#' packaging/runtime-libs | awk 'NF { sub(/\.so.*/, "", $1); print $1 }')
    mentioned=$(grep -aoE 'lib[A-Za-z0-9_+-]+\.so' "$out/draw" | sed 's/\.so$//' | sort -u)
    grep -qx libX11 <<<"$mentioned" || fail "library scan found nothing in $out/draw"
    for base in $mentioned; do
        grep -qx "$base" <<<"$listed" || missing+=("$base")
    done
    ((${#missing[@]} == 0)) || fail "add to packaging/runtime-libs: ${missing[*]}"
    echo "every library the binary names is listed"
}

# AC-2: no GLIBC_ symbol version above the floor.
check_glibc_floor() {
    local newest
    newest=$(objdump -T "$out/draw" | grep -o 'GLIBC_[0-9][0-9.]*' | sed 's/GLIBC_//' | sort -uV | tail -n 1)
    [[ -n $newest ]] || fail "no GLIBC_ versions found in $out/draw"
    [[ $(printf '%s\n%s\n' "$newest" "$glibc_floor" | sort -V | tail -n 1) == "$glibc_floor" ]] \
        || fail "binary needs GLIBC_$newest, above the $glibc_floor floor"
    echo "newest glibc symbol: GLIBC_$newest (floor $glibc_floor)"
}

build_id_of() { readelf -n "$1" | sed -n 's/^ *Build ID: *//p'; }

# AC-5: stripped binary under the size budget, linked to draw.debug by
# build id and .gnu_debuglink.
check_debug_symbols() {
    local shipped symbols size
    shipped=$(build_id_of "$out/draw")
    symbols=$(build_id_of "$out/draw.debug")
    [[ -n $shipped ]] || fail "shipped binary has no build id"
    [[ $shipped == "$symbols" ]] || fail "build ids differ: $shipped vs $symbols"
    readelf --string-dump=.gnu_debuglink "$out/draw" | grep -q 'draw\.debug' \
        || fail "shipped binary has no .gnu_debuglink to draw.debug"
    readelf -S "$out/draw" | grep -q '\.debug_info' && fail "shipped binary is not stripped"
    readelf -S "$out/draw.debug" | grep -q '\.debug_info' || fail "draw.debug has no debug info"
    size=$(stat -c %s "$out/draw")
    ((size < max_binary_bytes)) || fail "shipped binary is $size bytes, budget $max_binary_bytes"
    echo "build id $shipped, binary $size bytes"
}

# AC-6: desktop entry and metainfo validate.
check_desktop_files() {
    desktop-file-validate "packaging/$app_id.desktop"
    local metainfo=packaging/$app_id.metainfo.xml
    # EL8 has no appstreamcli; libappstream-glib's validator stands in.
    if command -v appstreamcli >/dev/null; then
        appstreamcli validate --no-net "$metainfo"
    else
        appstream-util validate-relax --nonet "$metainfo"
    fi
}

# AC-7: the tarball installs every file into a prefix and uninstalls all.
check_tarball_install() {
    local tmp expected installed left
    tmp=$(mktemp -d)
    tar -C "$tmp" -xzf "$tarball"
    local dir=$tmp/draw-$version-x86_64-linux
    for file in install.sh README.md CHANGELOG.md LICENSE; do
        [[ -f $dir/$file ]] || fail "tarball lacks $file"
    done
    expected=$(cd "$out/stage/usr" && find . -type f | sed 's|^\./||' | sort)
    HOME=$tmp "$dir/install.sh" --prefix "$tmp/prefix" >/dev/null
    installed=$(cd "$tmp/prefix" && find . -type f | sed 's|^\./||' | sort)
    [[ $installed == "$expected" ]] \
        || fail "install differs from the staging tree: $(diff <(echo "$expected") <(echo "$installed") | tr '\n' ' ')"
    grep -qx "Exec=$tmp/prefix/bin/draw" "$tmp/prefix/share/applications/$app_id.desktop" \
        || fail "installed desktop entry does not point at the binary"
    HOME=$tmp "$dir/install.sh" --prefix "$tmp/prefix" --uninstall >/dev/null
    left=$(find "$tmp/prefix" -type f)
    [[ -z $left ]] || fail "uninstall left: $left"
    rm -rf "$tmp"
    echo "tarball installs $(wc -l <<<"$expected") files and removes them all"
}

run_checks() {
    step "check: runtime libraries";  check_runtime_libs
    step "check: glibc floor";        check_glibc_floor
    step "check: debug symbols";      check_debug_symbols
    step "check: desktop files";      check_desktop_files
    step "check: tarball install";    check_tarball_install
}

if [[ ${1:-} == --check ]]; then
    run_checks
    exit 0
fi

flatpak=true
for arg in "$@"; do
    case "$arg" in
        --no-flatpak) flatpak=false ;;
        *) echo "unknown argument: $arg" >&2; exit 2 ;;
    esac
done

command -v podman >/dev/null || fail "podman is required"

image=draw-package:$(sha256sum packaging/Containerfile | cut -c1-12)
if ! podman image exists "$image"; then
    step "build image $image"
    podman build -t "$image" -f packaging/Containerfile packaging
fi

step "build in container"
podman run --rm \
    -v "$PWD:/src:Z" \
    -v draw-package-cargo:/opt/cargo/registry \
    -w /src \
    "$image" \
    bash -c 'packaging/build.sh && scripts/package.sh --check'

rm -rf dist
mkdir -p dist
cp "$tarball" "$out/draw_${version}-1_amd64.deb" "$out/draw-${version}-1.x86_64.rpm" \
    "$out/draw.debug" dist/

if $flatpak; then
    builder=()
    if command -v flatpak-builder >/dev/null; then
        builder=(flatpak-builder)
    elif flatpak info org.flatpak.Builder >/dev/null 2>&1; then
        builder=(flatpak run org.flatpak.Builder)
    fi
    if ((${#builder[@]})); then
        step "flatpak"
        "${builder[@]}" --user --install-deps-from=flathub --force-clean \
            --repo="$out/flatpak-repo" "$out/flatpak-build" \
            "packaging/flatpak/$app_id.yml"
        flatpak build-bundle "$out/flatpak-repo" "dist/draw-$version-x86_64.flatpak" "$app_id"
    else
        step "flatpak skipped: install flatpak-builder or org.flatpak.Builder to build the bundle"
    fi
fi

(cd dist && sha256sum -- * >SHA256SUMS)
step "packages in dist/"
ls -l dist
