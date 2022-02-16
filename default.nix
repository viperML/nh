{ lib, poetry2nix , python3, nvd, update-nix-fetchgit }:

poetry2nix.mkPoetryApplication rec {
  python = python3;

  src = ./.;
  projectDir = src;

  postFixup = ''
    substituteInPlace nh/deps.py \
        --replace 'nvd' '${nvd}/bin/nvd'
    substituteInPlace nh/deps.py \
        --replace 'update-nix-fetchgit' '${update-nix-fetchgit}/bin/update-nix-fetchgit'
  '';

  meta = with lib; {
    inherit (python.meta) platforms;
    description = "NH is yet another Nix cli Help utility";
    license = licenses.mit;
    homepage = "https://github.com/viperML/nh";
  };
}
