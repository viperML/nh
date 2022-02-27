## Running

```console
nix run github:viperML/nh -- --help
```

The environment variable `FLAKE` is used in multiple commands. This is intended to be the path to your NixOS's system flake, or a flake you use often.

## Installation

This a example installation to a NixOS system. Adapt accordingly if you want it in home-manager, a devShell, etc

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nh.url = "github:viperML/nh";
    nh.inputs.nixpkgs.follows = "nixpkgs";
    # ...
  };
  outputs = inputs @ {self, nixpkgs, ... }: {
    nixosConfigurations.my-host = nixpkgs.lib.nixosSystem rec {
      # ...
      system = "x86_64-linux";
      modules = [
        {
          environment.systemPackages = [inputs.nh.packages.${system}.nh];
          # Not necessary, but recommended to use your flake by default
          environment.variables.FLAKE = "/path/to/your/flake/root";
        }
      ];
    };
    # ...
  }
}
```

## Hacking

```console
git clone https://github.com/viperML/nh && cd nh
nix develop
python -m nh
```

## Todo's

- [x] Repl helper
- [x] NixOS update wrapper with nvd
- [x] Flake update helper (last update of inputs, recurse fetchFromGitHub)
- [ ] garbage-collect on steroids (hunt gc roots and prompt to remove them) (nix-du?)
- [ ] better search (maybe query search.nixos.org?)
