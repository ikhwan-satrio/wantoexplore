{
  description = "TeddyPicker — Tauri v2 file manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        version = "1.4.0";

        debUrl = "https://github.com/ikhwan-satrio/teddypicker/releases/download/v${version}/teddypicker_${version}_amd64.deb";

        deb = pkgs.fetchurl {
          url = debUrl;
          hash = "sha256:860f240cec5fc73903b80ec1aefbbe1d3f325a18bfc5749045cfdaa816298bce";
        };

        runtimeDeps = with pkgs; [
          webkitgtk_4_1
          gtk3
          glib
          libayatana-appindicator
          librsvg
          libsoup_3
          openssl
          sqlite
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = runtimeDeps;
          nativeBuildInputs = with pkgs; [
            pkg-config
            bun
            nodejs
            glib
            gsettings-desktop-schemas
            hicolor-icon-theme
          ];

          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath runtimeDeps}:$LD_LIBRARY_PATH"
          '';
        };

        packages.default = pkgs.stdenv.mkDerivation {
          pname = "teddypicker";
          inherit version;

          src = deb;

          nativeBuildInputs = with pkgs; [
            dpkg
            autoPatchelfHook
            makeWrapper
          ];

          buildInputs = runtimeDeps ++ (with pkgs; [
            glib
            gsettings-desktop-schemas
            hicolor-icon-theme
          ]);

          dontConfigure = true;
          dontBuild = true;

          installPhase = ''
            runHook preInstall

            dpkg -x $src unpacked

            mkdir -p $out/bin
            if [ -f unpacked/usr/bin/teddypicker ]; then
              cp unpacked/usr/bin/teddypicker $out/bin/.teddypicker-unwrapped
            else
              cp unpacked/usr/bin/app $out/bin/.teddypicker-unwrapped
            fi

            mkdir -p $out/share
            cp -r unpacked/usr/share/* $out/share/

            chmod +x $out/bin/.teddypicker-unwrapped

            wrapProgram $out/bin/.teddypicker-unwrapped \
              --prefix GDK_BACKEND : "wayland:x11" \
              --prefix XDG_CURRENT_DESKTOP : "GNOME" \
              --set LD_LIBRARY_PATH "${pkgs.lib.makeLibraryPath runtimeDeps}"

            ln -s $out/bin/.teddypicker-unwrapped $out/bin/teddypicker

            runHook postInstall
          '';

          meta = with pkgs.lib; {
            description = "File manager";
            homepage = "https://github.com/ikhwan-satrio/teddypicker";
            license = licenses.mit;
            platforms = [ "x86_64-linux" ];
            mainProgram = "teddypicker";
          };
        };
      }
    );
}
