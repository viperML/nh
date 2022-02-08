from importlib.resources import path
import pathlib


def find_flake(path_str: str):
    path = pathlib.Path(path_str)
    path.resolve()

    if not path.is_file():
        path = path / "flake.nix"

    if not path.exists():
        raise FileNotFoundError(f"{path} does not exist")
    else:
        return path
