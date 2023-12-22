{
  mkShell,
  rust-analyzer-unwrapped,
  rustfmt,
  clippy,
  nvd,
  nix-output-monitor,
  cargo,
  rustc,
  rustPlatform,
}:
mkShell {
  strictDeps = true;

  nativeBuildInputs = [
    cargo
    rustc

    rust-analyzer-unwrapped
    rustfmt
    clippy
    nvd
    nix-output-monitor
  ];

  buildInputs = [];

  env = {
    NH_NOM = "1";
    RUST_LOG = "nh=trace";
    RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
  };
}
