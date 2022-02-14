from pathlib import Path, PurePath


class nixfile(object):
    is_flake = None
    path = None

    def __init__(self, path_str: str):
        self.path = Path(path_str).resolve()

        # If we receive a folder, try to resolve the file containing
        if not self.path.is_file():
            flake_path = self.path / "flake.nix"
            default_path = self.path / "default.nix"

            if flake_path.exists():
                self.path = flake_path
            elif default_path.exists():
                self.path = default_path
            else:
                raise FileNotFoundError

        if self.path.name == "flake.nix":
            self.is_flake = True
        else:
            self.is_flake = False


def find_nixfiles(path_str) -> list[nixfile]:
    pass
