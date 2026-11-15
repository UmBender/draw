{
  description = "draw: minimal, fast, keyboard-first scratch canvas for competitive programming";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      # The toolchain, written once: Cargo.toml's rust-version is 1.99
      # (tests/packaging.rs checks they agree).
      rustVersion = "1.99.0";

      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forSystems = f: nixpkgs.lib.genAttrs systems (system: f system);

      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            self.overlays.default
          ];
        };

      # Libraries draw loads with dlopen, one per soname of
      # packaging/runtime-libs (roles required, gl, wayland, optional);
      # tests/packaging.rs fails when this list drifts from that file.
      runtimeLibs =
        pkgs: with pkgs; [
          { soname = "libX11.so.6"; pkg = libx11; }
          { soname = "libXi.so.6"; pkg = libxi; }
          { soname = "libxkbcommon.so.0"; pkg = libxkbcommon; }
          { soname = "libGL.so.1"; pkg = libglvnd; }
          { soname = "libEGL.so.1"; pkg = libglvnd; }
          { soname = "libwayland-client.so.0"; pkg = wayland; }
          { soname = "libwayland-egl.so.1"; pkg = wayland; }
          { soname = "libdecor-0.so.0"; pkg = libdecor; }
        ];

      libraryPath =
        pkgs:
        pkgs.lib.makeLibraryPath (pkgs.lib.unique (map (lib: lib.pkg) (runtimeLibs pkgs)));
    in
    {
      overlays.default = final: _prev: {
        draw =
          let
            toolchain = final.rust-bin.stable.${rustVersion}.default;
            rustPlatform = final.makeRustPlatform {
              cargo = toolchain;
              rustc = toolchain;
            };
            appId = "io.github.UmBender.draw";
          in
          rustPlatform.buildRustPackage {
            pname = "draw";
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
            src = self;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [ final.makeWrapper ];

            # The tests need a display and fonts of their own: the
            # repository's gates run them (scripts/check.sh).
            doCheck = false;

            postInstall = ''
              install -Dm644 packaging/${appId}.desktop \
                $out/share/applications/${appId}.desktop
              install -Dm644 packaging/icons/draw.svg \
                $out/share/icons/hicolor/scalable/apps/${appId}.svg
              for size in 48 128 256; do
                install -Dm644 packaging/icons/draw-$size.png \
                  $out/share/icons/hicolor/''${size}x$size/apps/${appId}.png
              done
              install -Dm644 packaging/${appId}.metainfo.xml \
                $out/share/metainfo/${appId}.metainfo.xml
              install -Dm644 LICENSE $out/share/licenses/draw/LICENSE
              install -Dm644 assets/fonts/OFL.txt $out/share/licenses/draw/OFL.txt
              install -Dm644 assets/fonts/GUST-FONT-LICENSE.txt \
                $out/share/licenses/draw/GUST-FONT-LICENSE.txt
              install -Dm644 README.md $out/share/doc/draw/README.md
              install -Dm644 CHANGELOG.md $out/share/doc/draw/CHANGELOG.md

              # The preflight and dlopen read LD_LIBRARY_PATH; NixOS has
              # no ld.so.cache. The GL drivers come from
              # /run/opengl-driver through libglvnd.
              wrapProgram $out/bin/draw \
                --prefix LD_LIBRARY_PATH : ${libraryPath final}:/run/opengl-driver/lib
            '';

            meta = {
              description = "Keyboard-first scratch canvas for competitive programming";
              license = final.lib.licenses.mit;
              mainProgram = "draw";
              platforms = systems;
            };
          };
      };

      packages = forSystems (system: rec {
        draw = (pkgsFor system).draw;
        default = draw;
      });

      apps = forSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/draw";
        };
      });

      devShells = forSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            packages =
              [
                pkgs.rust-bin.stable.${rustVersion}.default
                pkgs.pkg-config
                pkgs.xvfb-run
                pkgs.sway
                pkgs.grim
                pkgs.xdotool
              ];
            LD_LIBRARY_PATH = libraryPath pkgs;
          };
        }
      );

      checks = forSystems (system: {
        draw = self.packages.${system}.default;
      });
    };
}
