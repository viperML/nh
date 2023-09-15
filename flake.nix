{
  inputs = {
    # Not compatible with nixos-23.05
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs = inputs: let
    src = inputs.nix-filter.lib {
      root = inputs.self.outPath;
      include = [
        (inputs.nix-filter.lib.inDirectory "src")
        "Cargo.toml"
        "Cargo.lock"
        "build.rs"
      ];
    };
  in
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
      ];

      systems = [
        "aarch64-linux"
        "x86_64-linux"
      ];

      flake.nixosModules.default = import ./module.nix inputs.self;

      perSystem = {
        system,
        pkgs,
        config,
        ...
      }: {
        packages = {
          default = pkgs.callPackage ./default.nix {inherit src;};
          debug = pkgs.callPackage ./default.nix {
            inherit src;
            buildType = "debug";
          };
        };

        overlayAttrs.nh = config.packages.default;

        devShells.default = with pkgs;
          mkShell {
            # Shell with CC
            name = "nh-dev";
            RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
            NH_NOM = "1";
            packages = [
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer-unwrapped
              nvd
              nix-output-monitor
            ];
          };
      };
    };
}
