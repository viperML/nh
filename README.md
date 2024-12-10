<h1 align="center">nh</h1>

<h6 align="center">Because the name "yet-another-<u>n</u>ix-<u>h</u>elper" was too long to type...</h1>

## What does it do?

Nh is my own take at reimplementing some commands from the NixOS ecosystem. I aim
to provide more feature and better ergonomics than the existing commands.

Nh has serveral subcommands, such as:

- `os`, which reimplements `nixos-rebuild`, with a tree of builds, diff and
confirmation.
- `home`, which reimplements `home-manager`.
- `search`, a super-fast package searching tool (powered by a ElasticSearch
client).
- `clean`, my own take at cleaning GC roots from a NixOS system.

This wouldn't be possible with the programs that nh runs under the hood:

- Tree of builds with [nix-output-monitor](https://github.com/maralorn/nix-output-monitor).
- Visualization of the upgrade diff with [nvd](https://khumba.net/projects/nvd).
- And of course, all the [crates](./Cargo.toml) we depend on.

<p align="center">
  <img
    alt="nh feature showcase"
    src="./.github/screenshot.png"
    width="800px"
  >
</p>


## Installation

The latest stable version is available in Nixpkgs. This repository provides the
latest development version of nh.

```
nix shell nixpkgs#nh # stable
nix shell github:viperML/nh # dev
```


### NixOS

We provide a NixOS module that integrates `nh clean` as a service. To enable it,
set the following configuration:

```nix
{ config, pkgs, ... }:
{
  programs.nh = {
    enable = true;
    clean.enable = true;
    clean.extraArgs = "--keep-since 4d --keep 3";
    flake = "/home/user/my-nixos-config";
  };
}
```

Nh supports both Flakes and classical NixOS configurations:

- For flakes, the command is `nh os switch /path/to/flake`
- For a classical configuration, you must use `nh os switch -f '<nixpkgs/nixos>'`

You might want to check `nh os --help` for other values and the defaults from
environment variables.


#### Specialisations support

Nh is capable of detecting which specialisation you are running, so it runs the proper activation script.
To do so, you need to give nh some information of the spec that is currently running by writing its name to `/etc/specialisation`. The config would look like this:

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


# Status

[![Dependency status](https://deps.rs/repo/github/viperML/nh/status.svg)](https://deps.rs/repo/github/viperML/nh)

[![Packaging status](https://repology.org/badge/vertical-allrepos/nh.svg)](https://repology.org/project/unit/versions)

## Hacking

Just clone and `nix develop`. We also provide a `.envrc` for Direnv.
