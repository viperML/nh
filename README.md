<h1 align="center">nh</h1>

<h6 align="center">Because "yet-another-nix-helper" was too long...</h1>

<p align="center">
  <img
    alt="build: passing"
    src="https://img.shields.io/github/workflow/status/viperML/nh/build"
  >
  </a>
</p>


## What and why?

This tool is a set of commands that encapsulate commands that I used often.
This could be made with shell scripts and aliases, but using a python library to create the command makes it easier to maintain and debug.
```
Usage: nh [OPTIONS] COMMAND [ARGS]...

Options:
  --version  Show the version and exit.
  --help     Show this message and exit.

Commands:
  boot       Reimplementation of nixos-rebuild boot.
  gcr-clean  Find gcroots from a root directory, and delete them.
  repl       Load a flake into a nix repl
  search     Super fast search for packages.
  switch     Reimplementation of nixos-rebuild switch.
  test       Reimplementation of nixos-rebuild test.
```
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
