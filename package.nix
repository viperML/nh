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
}:
buildPythonPackage {
  name = "nh";
  src = lib.cleanSource ./.;

  propagatedBuildInputs= [
    click
    dateparser
    pyfzf
    colorama
    diskcache
    xdg
  ];
}
