{
  description = "NH is yet another Nix cli Help utility";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      pre-commit-hook = pkgs.writeShellScript "pre-commit" ''
        nix flake check
        # nix build .#nh --no-link
      '';
      nh-env = (pkgs.poetry2nix.mkPoetryEnv {
        projectDir = ./.;
      }).env.overrideAttrs (prev: {
        buildInputs = with pkgs; [
          poetry
        ];
        shellHook = ''
          ln -sf ${pre-commit-hook} .git/hooks/pre-commit
        '';
      });
    in
    rec {
      packages.nh = pkgs.callPackage ./default.nix { };
      apps.nh = flake-utils.lib.mkApp {
        drv = packages.nh;
        exePath = "/bin/nh";
      };
      defaultApp = apps.nh;
      devShell = nh-env;
    });
}
