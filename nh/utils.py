from pathlib import Path

import click

from .exceptions import FlakeNotInitialized


class NixFile(object):
    is_flake = False
    path = None
    updater = None
    has_fetchFromGitHub = False

    def __init__(self, path_str: str):
        self.path = Path(path_str).resolve()

        # If we receive a folder, try to resolve the file containing
        if self.path.is_dir():
            flake_path = self.path / "flake.nix"
            default_path = self.path / "default.nix"

            if flake_path.exists():
                self.path = flake_path
            elif default_path.exists():
                self.path = default_path
            else:
                raise FileNotFoundError

        if self.path.name == "flake.nix":
            # Probably rewrite this
            lockfile = (self.path / ".." / "flake.lock").resolve()
            if lockfile.exists():
                self.is_flake = True
            else:
                raise FlakeNotInitialized
        else:
            with open(self.path, "r") as f:
                if "fetchFromGitHub" in f.read():
                    self.has_fetchFromGitHub = True

    def __str__(self) -> str:
        return "nixfile: " + str(self.path)


def find_nixfiles(path: Path) -> list[NixFile]:
    result = []

    for f in path.rglob("*.nix"):
        try:
            result.append(NixFile(f))
        except FlakeNotInitialized:
            click.echo(f"Skipping {f} as it is a flake without lock file")

    return result


def cmd_print(cmd: list[str]) -> None:
    click.echo("$ " + " ".join(cmd))
