{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
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
  in {
    overlays.default = final: prev: {
      nh = final.callPackage ./package.nix {};
    };

    packages = forAllSystems (pkgs: rec {
      nh = pkgs.callPackage ./package.nix {};
      default = nh;
    });

    devShells = forAllSystems (pkgs: {
      default = pkgs.callPackage ./devshell.nix {};
    });

    nixosModules.default = import ./module.nix self;

    nixosConfigurations.check = let
      system = "x86_64-linux";
    in
      nixpkgs.lib.nixosSystem {
        modules = [
          nixpkgs.nixosModules.readOnlyPkgs
          {nixpkgs.pkgs = nixpkgs.legacyPackages.${system};}
          self.nixosModules.default
          {boot.isContainer = true;}
        ];
      };
  };
}
