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
      installShellCompletion $releaseDir/build/nh-*/out/nh.{bash,fish}
      installShellCompletion --zsh $releaseDir/build/nh-*/out/_nh
    '';
    postFixup = ''
      wrapProgram $out/bin/nh \
        --prefix PATH : ${lib.makeBinPath [nvd]}
    '';
  }
