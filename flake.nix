{
  description = "NH is yet another Nix cli Help utility";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs:
    inputs.flake-utils.lib.eachSystem ["x86_64-linux"] (system: let
      pkgs = inputs.nixpkgs.legacyPackages.${system};
      pre-commit-hook = pkgs.writeShellScript "pre-commit" ''
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
      nh-env =
        (pkgs.poetry2nix.mkPoetryEnv {
          projectDir = ./.;
        })
        .env
        .overrideAttrs (prev: {
          buildInputs = with pkgs; [
            poetry
            nvd
            update-nix-fetchgit
            fzf
          ];
          shellHook = ''
            ln -sf ${pre-commit-hook} .git/hooks/pre-commit
          '';
        });
    in rec {
      packages.nh = pkgs.callPackage ./default.nix {};
      apps.nh = inputs.flake-utils.lib.mkApp {
        drv = packages.nh;
        exePath = "/bin/nh";
      };
      defaultApp = apps.nh;
      devShell = nh-env;
    });
}
