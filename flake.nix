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
        "flake.lock"
        ".envrc"
        ".gitignore"
        (inputs.nix-filter.lib.matchExt "md")
        (inputs.nix-filter.lib.matchExt "json")
        (inputs.nix-filter.lib.matchExt "yaml")
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
        commonArgs = {
          inherit src;
          pname = cargo-toml.package.name;
          inherit (cargo-toml.package) version;
          cargoLock.lockFile = src + "/Cargo.lock";
          nativeBuildInputs = [
            pkgs.installShellFiles
          ];
          preFixup = ''
            installShellCompletion $releaseDir/build/nh-*/out/nh.{bash,fish}
            installShellCompletion --zsh $releaseDir/build/nh-*/out/_nh
          '';
        };

        wrapNh = drv:
          pkgs.symlinkJoin {
            inherit (drv) name pname version;
            paths = [drv];
            nativeBuildInputs = [pkgs.makeBinaryWrapper];
            postBuild = ''
              wrapProgram $out/bin/nh \
                --prefix PATH : ${with pkgs; lib.makeBinPath [nvd]}
            '';
          };
      in {
        packages = {
          _src = pkgs.symlinkJoin {
            name = "src";
            paths = [src];
          };

          _toolchain_dev = with inputs.fenix.packages.${system};
            combine [
              (stable.withComponents [
                "rustc"
                "cargo"
                "rust-src"
                "clippy"
                "rustfmt"
                "rust-analyzer"
              ])
            ];

          nh-dev =
            (pkgs.makeRustPlatform {
              cargo = config.packages._toolchain_dev;
              rustc = config.packages._toolchain_dev;
            })
            .buildRustPackage (
              commonArgs
              // {
                RUST_SRC_PATH = "${config.packages._toolchain_dev}/lib/rustlib/src/rust/library";
              }
            );

          nh = wrapNh (
            # use nixpkgs' rustPlatform without fenix for easy distribution
            pkgs.rustPlatform.buildRustPackage (
              commonArgs
              // {
                cargoBuildFlags = [
                  "--features=complete"
                ];
              }
            )
          );

          default = config.packages.nh;
        };

        devShells.extra = with pkgs;
          mkShellNoCC {
            name = "extra";
            packages = [
              nvd
            ];
          };
      };
    };
}
