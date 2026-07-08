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

        version = "1.2.0";

        debUrl = "https://github.com/ikhwan-satrio/teddypicker/releases/download/v${version}/teddypicker_${version}_amd64.deb";

        deb = pkgs.fetchurl {
          url = debUrl;
          hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
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
            wrapGAppsHook4
            glib
            gsettings-desktop-schemas
            hicolor-icon-theme
          ];
        };

        packages.default = pkgs.stdenv.mkDerivation {
          pname = "teddypicker";
          inherit version;

          src = deb;

          nativeBuildInputs = with pkgs; [
            dpkg
            autoPatchelfHook
            wrapGAppsHook4
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
              cp unpacked/usr/bin/teddypicker $out/bin/teddypicker
            else
              cp unpacked/usr/bin/app $out/bin/teddypicker
            fi

            mkdir -p $out/share
            cp -r unpacked/usr/share/* $out/share/

            chmod +x $out/bin/teddypicker

            runHook postInstall
          '';

          preFixup = ''
            gappsWrapperArgs+=(
              --prefix GDK_BACKEND : "wayland:x11"
              --prefix XDG_CURRENT_DESKTOP : "GNOME"
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath runtimeDeps}"
            )
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
