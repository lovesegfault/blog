{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";
  };

  outputs = { self, gitignore, nixpkgs, utils }:
    utils.lib.eachSystem [ "aarch64-darwin" "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs { inherit system; overlays = [ gitignore.overlay ]; };
      in
      {
        packages.default = with pkgs; stdenvNoCC.mkDerivation {
          name = "blog";
          src = gitignoreSource ./.;

          nativeBuildInputs = [ hugo ];

          dontConfigure = true;

          buildPhase = ''
            runHook preBuild
            hugo --minify
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p $out
            cp -r public/* $out
            runHook postInstall
          '';
        };

        devShells.default = self.packages.${system}.default.overrideAttrs (oldAttrs: {
          buildInputs = with pkgs; (oldAttrs.buildInputs or [ ]) ++ [
            git-lfs
            mdl
            nixpkgs-fmt
            pre-commit
            proselint
            rnix-lsp
            statix
            ltex-ls
          ];
        });
      });
}
