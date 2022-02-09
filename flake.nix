{
  description = "NH is yet another Nix cli Help utility";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      nh-env = (pkgs.poetry2nix.mkPoetryEnv {
        projectDir = ./.;
      }).env.overrideAttrs (prev: {
        buildInputs = with pkgs; [
          poetry
        ];
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
