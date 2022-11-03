{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    ...
  }: let
    src = inputs.nix-filter.lib {
      root = ./.;
      exclude = [
        (inputs.nix-filter.lib.matchExt "nix")
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
      }: let
        extraArgs = {
          nativeBuildInputs = [
            pkgs.installShellFiles
          ];
          preFixup = ''
            installShellCompletion $releaseDir/build/nh-*/out/nh.{bash,fish}
            installShellCompletion --zsh $releaseDir/build/nh-*/out/_nh
          '';
        };
      in {
        packages = {
          toolchain-dev = with inputs.fenix.packages.${system};
            combine [
              (complete.withComponents [
                "rustc"
                "cargo"
                "rust-src"
                "clippy"
                "rustfmt"
                "rust-analyzer"
              ])
            ];

          toolchain = with inputs.fenix.packages.${system};
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
            .buildRustPackage ({
                inherit src;
                pname = cargo-toml.package.name;
                inherit (cargo-toml.package) version;
                cargoLock.lockFile = src + "/Cargo.lock";
                RUST_SRC_PATH = "${config.packages.toolchain-dev}/lib/rustlib/src/rust/library";
              }
              // extraArgs);

          nh =
            (pkgs.makeRustPlatform {
              cargo = config.packages.toolchain;
              rustc = config.packages.toolchain;
            })
            .buildRustPackage ({
                inherit src;
                pname = cargo-toml.package.name;
                inherit (cargo-toml.package) version;
                cargoLock.lockFile = src + "/Cargo.lock";
                cargoBuildFlags = [
                  "--features=complete"
                ];
              }
              // extraArgs);

          default = config.packages.nh;
        };

        devShells.extra = with pkgs;
          mkShellNoCC {
            name = "extra";
            packages = [
              # rust-analyzer
            ];
          };
      };
    };
}
