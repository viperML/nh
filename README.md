# NH

_Because "yet another Nix Helper" was too long..._

## Running

```bash
nix run github:viperML/nh
```

## Hacking

```bash
nix develop
```

Or, a `.envrc` like this (the python env will be linked into `./.venv`):

```bash
use flake
local venv="$(dirname $(which python))/.."
local venv_resolved=$(builtin cd $venv; pwd)
ln -Tsf "$venv_resolved" .venv
```

## Todo's

- [ ] Repl helper
- [ ] Flake update helper (last update of inputs, etc)
- [ ] nix-build, nix build wrapper
- [ ] NixOS update wrapper with nvd
