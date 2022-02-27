import subprocess
from pathlib import Path
from typing import Union

import click
from colorama import Fore as F

from .exceptions import FlakeNotInitialized


class NixFile(object):
    is_flake = False
    path = None
    updater = None
    has_fetchFromGitHub = False

    def __init__(self, path: Union[Path, str]):
        self.path = Path(path).resolve()

        if not self.path.exists():
            raise FileNotFoundError

        # Canonicalize flake
        if self.path.name == "flake.nix":
            self.path = (self.path / "..").resolve()

        if self.path.is_dir():
            if (self.path / "flake.nix").exists():
                self.is_flake = True
            elif (self.path / "default.nix").exists():
                self.path = self.path / "default.nix"
            else:
                raise FileNotFoundError

        if self.is_flake and not (self.path / "flake.lock").exists():
            raise FlakeNotInitialized

        if not self.is_flake:
            with open(self.path, "r") as f:
                if "fetchFromGitHub" in f.read():
                    self.has_fetchFromGitHub = True

    def __str__(self) -> str:
        return str(self.path)


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


class SearchResult:
    def __init__(self, pname: str, flake: str) -> None:
        self.pname = pname

        try:
            self.description = (
                subprocess.check_output(
                    ["nix", "eval", f"{flake}#{pname}.meta.description"],
                    stderr=subprocess.DEVNULL,
                )
                .decode()
                .replace('"', "")
                .strip()
            )
        except subprocess.CalledProcessError:
            self.description = None

        try:
            self.version = (
                subprocess.check_output(
                    ["nix", "eval", f"{flake}#{pname}.version"],
                    stderr=subprocess.DEVNULL,
                )
                .decode()
                .replace('"', "")
                .strip()
            )
        except subprocess.CalledProcessError:
            self.version = None

        try:
            self.homepage = (
                subprocess.check_output(
                    ["nix", "eval", f"{flake}#{pname}.meta.homepage"],
                    stderr=subprocess.DEVNULL,
                )
                .decode()
                .replace('"', "")
                .strip()
            )
        except subprocess.CalledProcessError:
            self.homepage = None

        try:
            self.position = (
                subprocess.check_output(
                    ["nix", "eval", f"{flake}#{pname}.meta.position"],
                    stderr=subprocess.DEVNULL,
                )
                .decode()
                .replace('"', "")
                .strip()
                .split(":")[0]
            )
        except subprocess.CalledProcessError:
            self.position = None

    def print(self):
        print(f"{F.BLUE}{self.pname}{F.RESET}", end=" ")
        if self.version:
            print(f"({F.GREEN}{self.version}{F.RESET})")
        else:
            print()
        if self.description:
            print(f" {self.description}")
        if self.homepage:
            print(f" Homepage: {self.homepage}")
        if self.position:
            print(f" Source: {self.position}")
