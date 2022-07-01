{
  description = "NH is yet another Nix cli Help utility";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    flake-parts,
  }:
    flake-parts.lib.mkFlake {inherit self;} {
      systems = ["x86_64-linux" "aarch64-linux"];
      perSystem = {
        pkgs,
        self',
        ...
      }: let
      in {
        packages.default = pkgs.callPackage ./default.nix {};
        devShells.default = pkgs.mkShell {
          name = "nh-shell";
          inputsFrom = [
            self'.packages.default
            pkgs.poetry
          ];
          shellHook = ''
            set -x
            venv="$(cd $(dirname $(which python)); cd ..; pwd)"
            ln -Tsf "$venv" .venv
          '';
        };
      };
    };
}
