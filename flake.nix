{
  description = "NH is yet another Nix cli Help utility";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    flake-parts,
  }:
    flake-parts.lib.mkFlake {inherit self;} {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      flake.overlays.default = _: prev: {
        nh = prev.callPackage ./default.nix {};
      };
      perSystem = {
        pkgs,
        self',
        ...
      }: {
        packages =
          self.overlays.default null pkgs
          // {
            default = self'.packages.nh;
          };
        devShells.default = pkgs.mkShellNoCC {
          name = "nh-shell";
          packages = [
            pkgs.poetry
          ];
          inputsFrom = [
            self'.packages.nh
          ];
          shellHook = ''
            echo ">>> Linking python environment to $PWD/.venv"
            venv="$(cd $(dirname $(which python)); cd ..; pwd)"
            ln -Tsfv "$venv" .venv
          '';
        };
      };
    };
}
