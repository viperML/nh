{
  src ? ./.,
  rustPlatform,
  installShellFiles,
  makeWrapper,
  lib,
  nvd,
  use-nom ? true,
  nix-output-monitor,
  buildType ? "release"
}: let
  cargo-toml = builtins.fromTOML (builtins.readFile (src + "/Cargo.toml"));
  propagatedBuildInputs = [nvd] ++ lib.optionals use-nom [nix-output-monitor];
in
  rustPlatform.buildRustPackage {
    inherit src propagatedBuildInputs buildType;
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

    postFixup = ''
      wrapProgram $out/bin/nh --prefix PATH : ${lib.makeBinPath propagatedBuildInputs} ${lib.optionalString use-nom "--set-default NH_NOM 1"}
    '';

    meta.mainProgram = "nh";
  }
