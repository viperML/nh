{ lib, poetry2nix , python3 }:

poetry2nix.mkPoetryApplication rec {
  python = python3;

  src = ./.;
  projectDir = src;

  meta = with lib; {
    inherit (python.meta) platforms;
  };
}
