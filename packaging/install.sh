#!/bin/sh
# Installs draw from the unpacked tarball into a prefix (ADR-T27-2).
# Usage: ./install.sh [--prefix DIR] [--uninstall]
# The default prefix is ~/.local; --uninstall removes exactly the files
# an install puts there.
set -eu

app_id=io.github.UmBender.draw
prefix="${HOME:?HOME is not set}/.local"
uninstall=false

usage() { sed -n '2,5s/^# \{0,1\}//p' "$0"; }

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)
            [ $# -ge 2 ] || { usage >&2; exit 2; }
            prefix=$2; shift 2 ;;
        --prefix=*) prefix=${1#--prefix=}; shift ;;
        --uninstall) uninstall=true; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "install.sh: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

case "$prefix" in
    /*) ;;
    *) prefix="$(pwd)/$prefix" ;;
esac

here=$(cd "$(dirname "$0")" && pwd)

# Every file the install writes, relative to the prefix.
files="bin/draw
share/applications/$app_id.desktop
share/icons/hicolor/scalable/apps/$app_id.svg
share/icons/hicolor/48x48/apps/$app_id.png
share/icons/hicolor/128x128/apps/$app_id.png
share/icons/hicolor/256x256/apps/$app_id.png
share/metainfo/$app_id.metainfo.xml
share/licenses/draw/LICENSE
share/licenses/draw/OFL.txt
share/licenses/draw/GUST-FONT-LICENSE.txt
share/doc/draw/README.md
share/doc/draw/CHANGELOG.md"

if $uninstall; then
    for file in $files; do
        rm -f "$prefix/$file"
    done
    rmdir "$prefix/share/licenses/draw" "$prefix/share/doc/draw" 2>/dev/null || true
    echo "draw removed from $prefix"
    exit 0
fi

for file in $files; do
    mode=0644
    [ "$file" = bin/draw ] && mode=0755
    mkdir -p "$(dirname "$prefix/$file")"
    cp "$here/usr/$file" "$prefix/$file"
    chmod "$mode" "$prefix/$file"
done

# Launchers do not search ~/.local/bin: point the entry at the binary.
if [ "$prefix" != /usr ]; then
    desktop="$prefix/share/applications/$app_id.desktop"
    sed "s|^Exec=draw\$|Exec=$prefix/bin/draw|" "$desktop" > "$desktop.tmp"
    mv "$desktop.tmp" "$desktop"
    chmod 0644 "$desktop"
fi

echo "draw installed into $prefix"
case ":${PATH:-}:" in
    *":$prefix/bin:"*) ;;
    *) echo "note: $prefix/bin is not in PATH; start it from the launcher or as $prefix/bin/draw" ;;
esac
