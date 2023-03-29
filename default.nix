{
  src ? ./.,
  rustPlatform,
  installShellFiles,
  makeWrapper,
  lib,
  nvd,
}: let
  cargo-toml = builtins.fromTOML (builtins.readFile (src + "/Cargo.toml"));
in
  rustPlatform.buildRustPackage {
    inherit src;
    pname = cargo-toml.package.name;
    inherit (cargo-toml.package) version;
    cargoLock.lockFile = src + "/Cargo.lock";
    nativeBuildInputs = [
      installShellFiles
      makeWrapper
    ];
    cargoBuildFlags = [
      "--features=complete"
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
        --prefix PATH : ${lib.makeBinPath [nvd]}
    '';
  }
