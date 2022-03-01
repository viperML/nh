<h1 align="center">nh</h1>

<p align="center">
  <img
    alt="build: passing"
    src="https://img.shields.io/github/workflow/status/viperML/nh/build"
  >
  </a>
</p>

_Because "yet-another-**n**ix-**h**elper" was too long..._

## What and why?

This tool is a set of commands that encapsulate commands that I used often.
This could be made with shell scripts and aliases, but using a python library to create the command makes it easier to maintain and debug.
```
Usage: nh [OPTIONS] COMMAND [ARGS]...

Options:
  --version  Show the version and exit.
  --help     Show this message and exit.

Commands:
  boot       Like 'nixos-rebuild boot', but with a diff showing...
  gcr-clean  Find gcroots from a root directory, and delete them.
  repl       Load a flake into a nix repl
  search     Super fast search for packages (optionally interactive)
  switch     Like 'nixos-rebuild switch', but with a diff showing...
  test       Like 'nixos-rebuild test', but with a diff showing...
  update     Update a flake or any nix file containing fetchFromGitHub
```
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
```

## Todo's

- [x] Repl helper
- [x] NixOS update wrapper with nvd
- [x] Flake update helper (last update of inputs, recurse fetchFromGitHub)
- [ ] garbage-collect on steroids (hunt gc roots and prompt to remove them) (nix-du?)
- [ ] better search (maybe query search.nixos.org?)
