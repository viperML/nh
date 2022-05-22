{
  lib,
  poetry2nix,
  python3,
  nvd,
  update-nix-fetchgit,
  gnused,
  fzf,
}:
poetry2nix.mkPoetryApplication rec {
  python = python3;

  src = ./.;
  projectDir = src;

  propagatedBuildInputs = [
    nvd
    update-nix-fetchgit
    gnused
  ];

  meta = {
    inherit (python.meta) platforms;
    description = "NH is yet another Nix cli Help utility";
    license = lib.licenses.mit;
    homepage = "https://github.com/viperML/nh";
  };
}
