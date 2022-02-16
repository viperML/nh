# NH

_Because "yet another Nix Helper" was too long..._

## Running

```bash
nix run github:viperML/nh -- --help
```

## Hacking

```bash
git clone https://github.com/viperML/nh && cd nh
nix develop
python -m nh
```

## Requirements

- update-nix-fetchgit

## Todo's

- [x] Repl helper
- [x] Flake update helper (last update of inputs, recurse fetchFromGitHub)
- [ ] nix-build, nix build wrapper
- [ ] NixOS update wrapper with nvd
- [ ] garbage-collect on steroids (hunt gc roots and prompt to remove them) (nix-du?)
- [ ] nix-bundle integration?
- [ ] format nix files with nixkgs-fmt?
- [ ] better search (maybe query search.nixos.org?)
