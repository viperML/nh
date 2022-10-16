{
  lib,
  buildPythonPackage,
  #
  click,
  dateparser,
  pyfzf,
  colorama,
  diskcache,
  xdg,
  #
  nvd,
  fzf,
}:
buildPythonPackage {
  name = "nh";
  src = lib.cleanSource ./.;

  propagatedBuildInputs = [
    click
    dateparser
    pyfzf
    colorama
    diskcache
    xdg

    nvd
    fzf
  ];
}
