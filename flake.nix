{
  description = "NH is yet another Nix cli Help utility";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
    in
    rec {
      packages.nh = pkgs.callPackage ./default.nix { };
      defaultPackage = packages.nh;
      apps.nh = flake-utils.lib.mkApp {
        drv = packages.nh;
        exePath = "/bin/nh";
      };
      defaultApp = apps.nh;
    });
}
