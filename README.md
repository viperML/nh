# nh

_Because "yet-another-**n**ix-**h**elper" was too long..._

## What and why?

This tool is a set of commands that encapsulate commands that I used often.
This could be made with shell scripts and aliases, but using a python library to create the command makes it easier to maintain and debug.

## Running

```bash
nix run github:viperML/nh -- --help
```

The environment variable `FLAKE` is used in multiple commands. This is intended to be the path to your NixOS's system flake, or a flake you use often.

## Hacking

```bash
git clone https://github.com/viperML/nh && cd nh
nix develop
python -m nh
```

## Todo's

- [x] Repl helper
- [x] NixOS update wrapper with nvd
- [ ] Flake update helper (last update of inputs, recurse fetchFromGitHub)
- [ ] nix-build, nix build wrapper
- [ ] garbage-collect on steroids (hunt gc roots and prompt to remove them) (nix-du?)
- [ ] nix-bundle integration?
- [ ] format nix files with nixkgs-fmt?
- [ ] better search (maybe query search.nixos.org?)
