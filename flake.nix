{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";
  };

  outputs =
    { nixpkgs, utils, ... }:
    utils.lib.eachSystem [ "aarch64-darwin" "x86_64-linux" ] (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs.lib.fileset) difference toSource unions;

        source =
          paths:
          toSource {
            root = ./.;
            fileset = unions paths;
          };

        rust =
          crates: args:
          pkgs.rustPlatform.buildRustPackage (
            args
            // {
              version = "0.1.0";
              src = source [
                ./.cargo
                ./Cargo.lock
                ./Cargo.toml
                crates
              ];
              cargoLock.lockFile = ./Cargo.lock;
            }
          );

        forno = rust (difference ./crates ./crates/explainers/src/bin) {
          pname = "forno";
          cargoTestFlags = [ "--workspace" ];
        };

        programs = rust ./crates {
          pname = "programs";
          nativeBuildInputs = [ pkgs.lld ];
          auditable = false;
          doCheck = false;
          buildPhase = "cargo wasm --offline";
          installPhase = "install -Dm444 -t $out target/wasm32-unknown-unknown/wasm/*.wasm";
        };

        vendor = pkgs.callPackage ./nix/vendor.nix { };

        site =
          drafts:
          pkgs.runCommand "blog"
            {
              src = source [
                ./assets
                ./site.toml
                ./static
                (difference ./content ./content/drafts)
                (unions drafts)
              ];
              env.FORNO_VENDOR = vendor;
              env.FORNO_WASM = programs;
            }
            ''
              ln -s $src/* .
              ${forno}/bin/forno build --out $out ${pkgs.lib.optionalString (drafts != [ ]) "--drafts"}
            '';
      in
      {
        packages.default = site [ ];

        checks.specimen = site [ ./content/drafts/specimen ];

        devShells.default = pkgs.mkShell {
          env.FORNO_VENDOR = vendor;
          packages = with pkgs; [
            (writeShellScriptBin "forno" ''exec cargo run --package forno -- "$@"'')
            cargo
            clippy
            git-lfs
            lld
            ltex-ls-plus
            markdownlint-cli
            nil
            nixfmt
            pre-commit
            proselint
            rust-analyzer
            rustc
            rustfmt
            statix
          ];
        };
      }
    );
}
