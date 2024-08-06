{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    forAllSystems = function:
      nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
        # experimental
        "x86_64-darwin"
        "aarch64-darwin"
      ] (system: function nixpkgs.legacyPackages.${system});

    rev = self.shortRev or self.dirtyShortRev or "dirty";
  in {
    overlays.default = final: prev: {
      nh = final.callPackage ./package.nix {
        inherit rev;
      };
    };

    packages = forAllSystems (pkgs: rec {
      nh = pkgs.callPackage ./package.nix {
        inherit rev;
      };
      default = nh;
    });

    devShells = forAllSystems (pkgs: {
      default = pkgs.callPackage ./devshell.nix {};
    });

    nixosModules.default = import ./module.nix;
  };
}
