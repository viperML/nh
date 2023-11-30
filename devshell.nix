{
  mkShell,
  rust-analyzer,
  rustfmt,
  clippy,
  nvd,
  nix-output-monitor,
  cargo,
  rustc,
}:
mkShell {
  strictDeps = true;

  nativeBuildInputs = [
    cargo
    rustc

    rust-analyzer
    rustfmt
    clippy
    nvd
    nix-output-monitor
  ];

  buildInputs = [];

  env = {
    NH_NOM = "1";
  };
}
