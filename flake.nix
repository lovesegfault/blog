{
  description = "lovesegfault's CV";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";
  };

  outputs = { self, gitignore, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; overlays = [ gitignore.overlay ]; };
      in
      {
        defaultPackage = with pkgs; stdenv.mkDerivation {
          name = "blog";
          src = gitignoreSource ./.;

          nativeBuildInputs = [ hugo ];

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

        devShell = self.defaultPackage.${system}.overrideAttrs (oldAttrs: {
          buildInputs = with pkgs; (oldAttrs.buildInputs or [ ]) ++ [
            git-lfs
            mdl
            nix-linter
            nixpkgs-fmt
            pre-commit
            proselint
            rnix-lsp
          ];
        });
      });
}
