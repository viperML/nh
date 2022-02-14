from path import Path


class nixfile(object):
    is_flake = None
    path = None

    def __init__(self, path_str: str):
        self.path = Path(path_str).abspath()

        # If we receive a folder, try to resolve the file containing
        if not self.path.isfile():
            flake_path = self.path / "flake.nix"
            default_path = self.path / "default.nix"

            if flake_path.exists():
                self.path = flake_path
            elif default_path.exists():
                self.path = default_path
            else:
                raise FileNotFoundError

        if self.path.basename() == "flake.nix":
            self.is_flake = True
        else:
            self.is_flake = False


pass
