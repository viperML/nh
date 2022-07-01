{
  lib,
  poetry2nix,
  python3,
  nvd,
  fzf,
}:
poetry2nix.mkPoetryApplication rec {
  python = python3;

  src = ./.;
  projectDir = src;

  propagatedBuildInputs = [
    nvd
    fzf
  ];

  meta = {
    inherit (python.meta) platforms;
    description = "NH is yet another Nix cli Help utility";
    license = lib.licenses.mit;
    homepage = "https://github.com/viperML/nh";
  };
}
