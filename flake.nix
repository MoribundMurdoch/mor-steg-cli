{
  description = "MorSteg CLI - noob-friendly age + steghide helper";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system:
          f (import nixpkgs { inherit system; })
        );
    in
    {
      packages = forAllSystems (pkgs:
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "mor-steg";
            version = "0.2.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = [
              pkgs.makeWrapper
            ];

            installPhase = ''
              runHook preInstall

              install -Dm755 target/release/mor-steg $out/bin/mor-steg

              if [ -f packaging/arch/mor-steg-sandboxed ]; then
                install -Dm755 packaging/arch/mor-steg-sandboxed $out/bin/mor-steg-sandboxed
              fi

              if [ -f packaging/arch/mor-steg.desktop ]; then
                install -Dm644 packaging/arch/mor-steg.desktop $out/share/applications/mor-steg.desktop
              fi

              if [ -f packaging/arch/mor-steg-sandboxed.desktop ]; then
                install -Dm644 packaging/arch/mor-steg-sandboxed.desktop $out/share/applications/mor-steg-sandboxed.desktop
              fi

              if [ -f assets/mor-steg.png ]; then
                install -Dm644 assets/mor-steg.png $out/share/icons/hicolor/256x256/apps/mor-steg.png
              fi

              install -Dm644 README.md $out/share/doc/mor-steg/README.md
              install -Dm644 LICENSE $out/share/licenses/mor-steg/LICENSE

              wrapProgram $out/bin/mor-steg \
                --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.age pkgs.steghide ]}

              if [ -f $out/bin/mor-steg-sandboxed ]; then
                wrapProgram $out/bin/mor-steg-sandboxed \
                  --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.bubblewrap pkgs.age pkgs.steghide ]}
              fi

              runHook postInstall
            '';

            meta = with pkgs.lib; {
              description = "Noob-friendly CLI front-end for age-encrypted steganography with steghide";
              homepage = "https://github.com/MoribundMurdoch/mor-steg-cli";
              license = licenses.unlicense;
              platforms = platforms.linux;
              mainProgram = "mor-steg";
            };
          };
        });

      apps = forAllSystems (pkgs:
        {
          default = {
            type = "app";
            program = "${self.packages.${pkgs.system}.default}/bin/mor-steg";
          };
        });
    };
}
