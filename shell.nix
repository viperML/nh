{ pkgs ? import <nixpkgs> {} }:
let
  poetry-env = pkgs.poetry2nix.mkPoetryEnv {
    projectDir = ./.;
  };
in poetry-env.env.overrideAttrs (prev: {
  buildInputs = with pkgs; [
    poetry
  ];
  # shellHook = ''
  #   mkdir -p .env
  #   ln -sf ${poetry-env} .env/python
  # '';
})
