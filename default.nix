{ lib, poetry2nix , python3, nvd, update-nix-fetchgit, gnused }:

poetry2nix.mkPoetryApplication rec {
  python = python3;

  src = ./.;
  projectDir = src;

  prePatch = ''
    ${gnused}/bin/sed -i "s#nvd#${nvd}/bin/nvd#g" nh/deps.py
    ${gnused}/bin/sed -i "s#update-nix-fetchgit#${update-nix-fetchgit}/bin/update-nix-fetchgit#g" nh/deps.py
  '';

  meta = {
    inherit (python.meta) platforms;
    description = "NH is yet another Nix cli Help utility";
    license = lib.licenses.mit;
    homepage = "https://github.com/viperML/nh";
  };
}
