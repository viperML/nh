# NH

_Because "yet another Nix Helper" was too long..._

## Running

```bash
nix run github:viperML/nh
```

## Hacking

```bash
git clone https://github.com/viperML/nh && cd nh
nix develop
python -m nh
```

A `.envrc` like this can be convenient (the python env will be linked into `./.venv`):

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
