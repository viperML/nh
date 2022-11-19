<h1 align="center">nh</h1>

<h6 align="center">Because the name "yet-another-nix-helper" was too long to type...</h1>

<p align="center">
  <img
    alt="build: passing"
    src="https://img.shields.io/github/workflow/status/viperML/nh/build"
  >
  </a>
</p>


## What does it do?

NH reimplements some basic NixOS commands, namely nixos-rebuild and nix-collect garbage. It supports building and properly activating a NixOS config that uses [specialisations](https://search.nixos.org/options?channel=unstable&show=specialisation).

It also includes `nvd` to show a pretty diff of the transaction, adds confirmations prompts and lets you pre-configure the path to your NixOS config.

To-do list of features:

- [x] Reimplement `nixos-rebuild {switch,boot,text}`
- [ ] Reimplement `nix-collect-garbage`
- [ ] Reimplement `home-manager switch`
- [ ] Reimplement `nix search`

## Installation

### I just want the CLI

`nh` is in this flake's `packages.<system>.default` output, so throw it into your config however you want. Feel free to override this flake's nixpkgs by using `nh.inputs.nixpkgs.follows`.

### Configure FLAKE env variable

`nh` uses the `FLAKE` environment variable as a default, so you don't have to pass the path to your flake for every command.

For NixOS, configuring it could be as simple as:

```nix
environment.sessionVariables.FLAKE = "/home/ayats/Documents/dotfiles";
```

### Configure specialisations

NH is capable of detecting which spec you are running, so it runs the proper activation script.
To do so, you need to give NH some information of the spec that is currently running by writing its name to `/etc/specialisation`. The config would look like this:

```nix
{...}: {
    specialisation."foo".configuration = {
        environment.etc."specialisation".text = "foo";
    };

    specialisation."bar".configuration = {
        environment.etc."specialisation".text = "bar";
    };
}
```

<details>
<summary> Why are specs broken with `nixos-rebuild` ? </summary>

To understand why `nixos-rebuild` doesn't work, we must know that it is just a shell wrapper around a more fundamental script from NixOS: `<toplevel package>/bin/switch-to-configuration` [^1].

[^1]: The toplevel package is what you can build with `nix build /flake#nixosConfiguration.HOSTNAME.config.system.build.toplevel`, and what sits on `/run/current-system`, `/run/booted-system` and `/nix/var/nix/profiles/system`.

> at the time of this writing

</details>

## Hacking

If you use [direnv](https://direnv.net/), just allow the `.envrc`.

Otherwise, `nix develop .#nh-dev`
