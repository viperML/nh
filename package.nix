{
  rustPlatform,
  installShellFiles,
  makeWrapper,
  lib,
  nvd,
  use-nom ? true,
  nix-output-monitor,
  buildType ? "release",
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  propagatedBuildInputs = [nvd] ++ lib.optionals use-nom [nix-output-monitor];
in
  rustPlatform.buildRustPackage {
    inherit propagatedBuildInputs buildType;
    pname = cargoToml.package.name;
    inherit (cargoToml.package) version;
    cargoLock.lockFile = ./Cargo.lock;

    src = lib.fileset.toSource {
      root = ./.;
      fileset =
        lib.fileset.intersection
        (lib.fileset.fromSource (lib.sources.cleanSource ./.))
        (lib.fileset.unions [
          ./src
          ./Cargo.toml
          ./Cargo.lock
        ]);
    };

    nativeBuildInputs = [
      installShellFiles
      makeWrapper
    ];

    preFixup = ''
      mkdir completions
      $out/bin/nh completions --shell bash > completions/nh.bash
      $out/bin/nh completions --shell zsh > completions/nh.zsh
      $out/bin/nh completions --shell fish > completions/nh.fish

      installShellCompletion completions/*
    '';

    postFixup = ''
      wrapProgram $out/bin/nh \
        --prefix PATH : ${lib.makeBinPath propagatedBuildInputs} \
        ${lib.optionalString use-nom "--set-default NH_NOM 1"}
    '';

    meta.mainProgram = "nh";

    strictDeps = true;
  }
