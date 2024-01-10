[![dependency status](https://deps.rs/repo/github/viperML/nh/status.svg)](https://deps.rs/repo/github/viperML/nh)

<h1 align="center">nh</h1>

<h6 align="center">Because the name "yet-another-<u>n</u>ix-<u>h</u>elper" was too long to type...</h1>

## What does it do?

NH reimplements some basic nix commands. Adding functionality on top of the existing solutions, like nixos-rebuild, home-manager cli or nix itself.

As the main features:
- Tree of builds with [nix-output-manager](https://github.com/maralorn/nix-output-monitor)
- Visualization of the upgrade diff with [nvd](https://gitlab.com/khumba/nvd)
- Asking for confirmation before performing activation

<p align="center">
  <img
    alt="build: passing"
    src="./.github/screenshot.png"
    width="800px"
  >
</p>


## Installation

### Nixpkgs

nh is available in nixpkgs:

- NixOS search: https://search.nixos.org/packages?channel=unstable&query=nh
- Hydra status:
  - x86_64-linux: https://hydra.nixos.org/job/nixos/trunk-combined/nixpkgs.nh.x86_64-linux
  - aarch64-linux: https://hydra.nixos.org/job/nixos/trunk-combined/nixpkgs.nh.aarch64-linux

### Flake

If you want to get the latest nh version not published to nixpkgs, you can use the flake

```nix
{
  inputs.nh = {
    url = "github:viperML/nh";
    inputs.nixpkgs.follows = "nixpkgs"; # override this repo's nixpkgs snapshot
  };
}
```

Then, include it in your `environment.systemPackages` or `home.packages` by referencing the input:
```
inputs.nh.packages.<system>.default
```


### Configure **FLAKE** env variable

nh uses the `FLAKE` env variable as a default for `os` and `home`. This is a shorthand for `--flake` in other commands. This saves typing it every time.

For NixOS, configuring it could be as simple as:

```
environment.sessionVariables.FLAKE = "/home/ayats/Documents/dotfiles";
```

### NixOS module

The nh NixOS modules provides an garbage collection alternative to the default one. Currently you can only get it through the flake.

```nix
nixosConfigurations.foo = nixpkgs.lib.nixosSystem {
  modules = [
    inputs.nh.nixosModules.default
    {
      nh = {
        enable = true;
        clean.enable = true;
        clean.extraArgs = "--keep-since 4d --keep 3";
      };
    }
  ];
}
```

### Configure specialisations

NH is capable of detecting which specialisation you are running, so it runs the proper activation script.
To do so, you need to give NH some information of the spec that is currently running by writing its name to `/etc/specialisation`. The config would look like this:

```nix
{config, pkgs, ...}: {
  specialisation."foo".configuration = {
    environment.etc."specialisation".text = "foo";
    # ..rest of config
  };

  specialisation."bar".configuration = {
    environment.etc."specialisation".text = "bar";
    # ..rest of config
  };
}
```

### Configure the **NH_NOM** env variable

By default nh uses nix-output-monitor (nom) to show the build log. This can be disabled either by:

- Exporting the environment variable `NH_NOM=0`
- Overriding the package: `nh.override { use-nom = false; }`

## Hacking

Just `nix develop`

[^1]: At the time of this writing.

[^2]: The toplevel package is what you can build with `nix build /flake#nixosConfiguration.HOSTNAME.config.system.build.toplevel`, and what sits on `/run/current-system`, `/run/booted-system` and `/nix/var/nix/profiles/system`.