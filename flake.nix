{
  description = "NH is yet another Nix cli Help utility";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    flake-parts.url = "github:hercules-ci/flake-parts";
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

      perSystem = {
        pkgs,
        config,
        system,
        ...
      }: {
        packages = rec {
          nh = pkgs.python3.pkgs.callPackage ./package.nix {};
          default = nh;
        };

        devShells.extra = pkgs.mkShellNoCC {
          name = "nh";
          packages = [
            pkgs.poetry
          ];
        };
      };
    };
}
