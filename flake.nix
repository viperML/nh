{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
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
      systems = [
        "aarch64-linux"
        "x86_64-linux"
      ];

      perSystem = {
        system,
        pkgs,
        config,
        ...
      }: {
        packages = {
          default = pkgs.callPackage ./default.nix {inherit src;};
        };

        devShells.default = with pkgs;
          mkShell {
            # Shell with CC
            name = "nh-dev";
            RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
            packages = [
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer-unwrapped
              nvd
            ];
          };
      };
    };
}
