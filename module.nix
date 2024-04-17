builtins.throw ''
  github:viperML/nh: the NixOS module has been upstreamed into nixpkgs

  https://github.com/NixOS/nixpkgs/pull/294923

  To migrate, please replace the import to this flake's module:
  >>> inputs.nh.nixosModules.default
  With just swapping the nh package:
  <<< { programs.nh.package = inputs.nh.packages.x86_64-linux.default; }

  The nh options are now in the programs.* namespace
  You may need to adjust nh.enable -> programs.nh.enable , etc.
''
