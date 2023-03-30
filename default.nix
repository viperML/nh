{
  src ? ./.,
  rustPlatform,
  installShellFiles,
  makeWrapper,
  lib,
  nvd,
  use-nom ? true,
  nix-output-monitor,
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

    preFixup = ''
      mkdir completions
      $out/bin/nh completions --shell bash > completions/nh.bash
      $out/bin/nh completions --shell zsh > completions/nh.zsh
      $out/bin/nh completions --shell fish > completions/nh.fish

      installShellCompletion completions/*
    '';

    postFixup =
      if use-nom
      then ''
        wrapProgram $out/bin/nh \
          --prefix PATH : ${lib.makeBinPath [
          nvd
          nix-output-monitor
        ]} \
        --set-default NH_NOM 1
      ''
      else ''
        wrapProgram $out/bin/nh \
          --prefix PATH : ${lib.makeBinPath [
          nvd
        ]}

      '';
  }
