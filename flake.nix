{
  description = "NH is yet another Nix cli Help utility";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = {
    self,
    nixpkgs,
  }: let
    pkgsFor = nixpkgs.legacyPackages;
    genSystems = nixpkgs.lib.genAttrs [
      "x86_64-linux"
    ];
  in {
    packages = genSystems (system: {
      default = pkgsFor.${system}.callPackage self {};
    });

    devShells = genSystems (system: {
      default = with pkgsFor.${system}; let
        pre-commit-hook = writeShellScript "pre-commit" ''
          set -ux
          find . -name \*.py -exec black {} \;
          find . -name \*.py -exec mypy {} \;
          flake8 --max-line-length=99
          nix flake check
          nix build .#nh --no-link
          my_nh=`nix eval --raw .#nh.outPath`
          echo "\`\`\`" > doc/01_README.md
          $my_nh/bin/nh --help >> doc/01_README.md
          echo "\`\`\`" >> doc/01_README.md
          cat doc/*_README.md > README.md
          git add .
        '';
      in
        (poetry2nix.mkPoetryEnv {
          projectDir = self;
        })
        .env
        .overrideAttrs (prevAttrs: {
          buildInputs = [
            poetry
            nvd
            update-nix-fetchgit
            fzf
          ];
          shellHook = ''
            ln -sf ${pre-commit-hook} .git/hooks/pre-commit
          '';
        });
    });
  };
}
