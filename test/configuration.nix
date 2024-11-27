{ pkgs, ... }:
{
  environment.systemPackages = [
    pkgs.hello
  ];
}
