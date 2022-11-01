{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    flake-parts.url = "github:hercules-ci/flake-parts";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs = {
    self,
    nixpkgs,
    flake-parts,
    naersk,
    fenix,
    nix-filter,
  }: let
    src = nix-filter.lib {
      root = ./.;
      exclude = [
        (nix-filter.lib.matchExt "nix")
      ];
    };
    cargo-toml = builtins.fromTOML (builtins.readFile (src + "/Cargo.toml"));
  in
    flake-parts.lib.mkFlake {inherit self;} {
      systems = [
        "aarch64-linux"
        "x86_64-linux"
        "x86_64-darwin"
      ];

      perSystem = {
        system,
        pkgs,
        config,
        ...
      }: {
        packages = {
          toolchain-dev = with fenix.packages.${system};
            combine [
              (complete.withComponents [
                "rustc"
                "cargo"
                "rust-src"
                "clippy"
                "rustfmt"
              ])
            ];

          toolchain = with fenix.packages.${system};
            combine [
              (complete.withComponents [
                "rustc"
                "cargo"
              ])
            ];

          nh-dev =
            (pkgs.makeRustPlatform {
              cargo = config.packages.toolchain-dev;
              rustc = config.packages.toolchain-dev;
            })
            .buildRustPackage {
              inherit src;
              pname = cargo-toml.package.name;
              inherit (cargo-toml.package) version;
              cargoLock.lockFile = src + "/Cargo.lock";
              RUST_SRC_PATH = "${config.packages.toolchain-dev}/lib/rustlib/src/rust/library";
            };

          nh =
            (pkgs.makeRustPlatform {
              cargo = config.packages.toolchain;
              rustc = config.packages.toolchain;
            })
            .buildRustPackage {
              inherit src;
              pname = cargo-toml.package.name;
              inherit (cargo-toml.package) version;
              cargoLock.lockFile = src + "/Cargo.lock";
            };

            default = config.packages.nh;
        };

        devShells.extra = with pkgs;
          mkShellNoCC {
            name = "extra";
            packages = [
              rust-analyzer
            ];
          };
      };
    };
}
