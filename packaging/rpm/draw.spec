%global app_id io.github.UmBender.draw

Name:           draw
Version:        1.6.0
Release:        1%{?dist}
Summary:        Minimal, fast, keyboard-first scratch canvas for competitive programming

License:        MIT AND OFL-1.0 AND LPPL-1.3c
URL:            https://github.com/UmBender/draw
Source0:        %{url}/archive/refs/tags/v%{version}.tar.gz
Source1:        draw-%{version}-vendor.tar.gz

BuildRequires:  cargo
BuildRequires:  rust

Requires:       libX11
Requires:       libXi
Requires:       libxkbcommon
Requires:       libglvnd-glx
Requires:       libglvnd-egl
Recommends:     libwayland-client
Recommends:     libwayland-egl

%description
Minimal, fast, keyboard-first scratch canvas for competitive programming
with freehand smoothing, graphs, grids, coordinate planes, and inline TeX.

%prep
%autosetup -n draw-%{version}
tar -xzf %{SOURCE1}
mkdir -p .cargo
cat > .cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

%build
cargo build --release --locked --offline

%install
install -Dm755 target/release/draw %{buildroot}%{_bindir}/draw
install -Dm644 packaging/%{app_id}.desktop \
    %{buildroot}%{_datadir}/applications/%{app_id}.desktop
install -Dm644 packaging/%{app_id}.metainfo.xml \
    %{buildroot}%{_datadir}/metainfo/%{app_id}.metainfo.xml
install -Dm644 packaging/icons/%{app_id}.svg \
    %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/%{app_id}.svg
for size in 48 128 256; do
    install -Dm644 packaging/icons/%{app_id}-${size}.png \
        %{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps/%{app_id}.png
done

%files
%license LICENSE assets/fonts/OFL.txt assets/fonts/GUST-FONT-LICENSE.txt
%doc README.md CHANGELOG.md
%{_bindir}/draw
%{_datadir}/applications/%{app_id}.desktop
%{_datadir}/metainfo/%{app_id}.metainfo.xml
%{_datadir}/icons/hicolor/*/apps/%{app_id}.*

%changelog
* Wed Nov 18 2026 UmBender <https://github.com/UmBender/draw> - 1.6.0-1
- Initial Fedora COPR RPM spec
