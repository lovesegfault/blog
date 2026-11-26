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

        fonts = pkgs.callPackage ./nix/fonts.nix { };

        # What posts can import from `@preview`.
        typst = pkgs.typst.withPackages (it: [ it.cetz_0_5_2 ]);

        # Graphviz measures labels with whatever font it finds, so it is left the one that they are set in.
        dot = pkgs.writeShellScript "dot" ''
          export FONTCONFIG_FILE=${pkgs.writeText "fonts.conf" "<fontconfig><dir>${fonts}/mono</dir></fontconfig>"}
          exec ${pkgs.graphviz}/bin/dot "$@"
        '';

        # For forno, which takes one path from each, and for tinymist.
        env = {
          GRAPHVIZ_DOT = dot;
          TYPST_FONT_PATHS = fonts;
          TYPST_PACKAGE_CACHE_PATH = "${typst}/lib/typst/packages";
        };

        site =
          drafts:
          pkgs.runCommand "blog"
            {
              src = source [
                ./assets
                ./blog.typ
                ./site.toml
                ./static
                (difference ./content ./content/drafts)
                (unions drafts)
              ];
              env = env // {
                FORNO_WASM = programs;
              };
            }
            ''
              ln -s $src/* .
              ${forno}/bin/forno build --out $out ${pkgs.lib.optionalString (drafts != [ ]) "--drafts"}
            '';
      in
      {
        packages = {
          inherit forno programs;
          default = site [ ];
        };

        checks.specimen = site [ ./content/drafts/specimen ];

        devShells.default = pkgs.mkShell {
          inherit env;
          shellHook = "[ ! -e site.toml ] || pre-commit install > /dev/null";
          packages = with pkgs; [
            (writeShellScriptBin "forno" ''exec cargo run --package forno -- "$@"'')
            cargo
            clippy
            git-lfs
            lld
            ltex-ls-plus
            nil
            nixfmt
            pre-commit
            proselint
            rust-analyzer
            rustc
            rustfmt
            statix
            tinymist
            typstyle
          ];
        };
      }
    );
}
