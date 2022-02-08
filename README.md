## Building

Recommended `.envrc`:

```bash
use nix
local venv="$(dirname $(which python))/.."
local venv_resolved=$(builtin cd $venv; pwd)
ln -Tsf "$venv_resolved" .venv
```
