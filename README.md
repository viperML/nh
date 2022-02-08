# NH

_Because "yet another Nix Helper" was too long..._

## Running

```bash
nix run github:viperML/nh
```

## Hacking

Recommended `.envrc`:

```bash
use nix
local venv="$(dirname $(which python))/.."
local venv_resolved=$(builtin cd $venv; pwd)
ln -Tsf "$venv_resolved" .venv
```

## Todo's

- [ ] Repl helper
- [ ] Flake update helper (last update of inputs, etc)
- [ ] nix-build, nix build wrapper
- [ ] NixOS update wrapper with nvd
