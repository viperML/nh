## Running

```console
nix run github:viperML/nh -- --help
```

The environment variable `FLAKE` is used in the commands `switch` `boot` `test` and `repl`. This is meant to be the path to your flake. Although you can pass `--flake /path/to/flake`, this makes it more convenient.

## Installation

This a example installation to a NixOS system or Home-manager.

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    nh.url = "github:viperML/nh";
    nh.inputs.nixpkgs.follows = "nixpkgs";
    # ...
  };
  outputs = inputs @ {self, nixpkgs, ... }: {
    nixosConfigurations.my-host = nixpkgs.lib.nixosSystem {
      # ...
      modules = [
        {
          environment.systemPackages = [inputs.nh.packages."x86_64-linux".default];
          # environment.variables.FLAKE = "/path/to/your/flake";
        }
      ];
    };

    homeConfigurations.my-user = home-manager.lib.homeManagerConfiguration {
      # ...
      modules = [
        {
          home.packages = [inputs.nh.packages."x86_64-linux".default];
          # home.sessionVariables.FLAKE = "/path/to/your/flake";
        }
      ];
    };
  };
}
```

## Hacking

```console
git clone https://github.com/viperML/nh && cd nh
nix develop
```
