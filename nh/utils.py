import concurrent.futures
import json
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Union
import os

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


def nix_eval(query: str) -> str:
    try:
        result = subprocess.check_output(
            ["nix", "eval", "--raw", query],
            stderr=subprocess.DEVNULL,
        ).decode()

        if "meta.position" in query:
            result = result.split(":")[0]
        return result
    except subprocess.CalledProcessError:
        return ""


class SearchResult:
    def __init__(self, pname: str, flake: str):
        self.pname = pname

        with concurrent.futures.ThreadPoolExecutor() as executor:
            futures = dict()
            futures["description"] = executor.submit(
                nix_eval, f"{flake}#{pname}.meta.description"
            )
            futures["version"] = executor.submit(nix_eval, f"{flake}#{pname}.version")
            futures["homepage"] = executor.submit(
                nix_eval, f"{flake}#{pname}.meta.homepage"
            )
            futures["position"] = executor.submit(
                nix_eval, f"{flake}#{pname}.meta.position"
            )

        self.description = futures["description"].result()
        self.version = futures["version"].result()
        self.homepage = futures["homepage"].result()
        self.position = futures["position"].result()

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


class GCRoot:
    def __init__(self, source: Union[Path, str], destination: Union[Path, str]):
        self.source = Path(source)
        self.destination = Path(destination)

        self.path_info = json.loads(
            subprocess.check_output(
                ["nix", "path-info", "-hS", "--json", str(self.source)]
            ).decode()
        )[0]

        self.registration_time = datetime.fromtimestamp(
            self.path_info["registrationTime"]
        )

    def remove(self) -> None:
        os.remove(self.destination)


def find_gcroots(root) -> list[GCRoot]:
    raw_lines = (
        subprocess.check_output(["nix-store", "--gc", "--print-roots"])
        .decode()
        .split("\n")
    )

    result = list()

    for line in raw_lines:
        if line and "censored" not in line:
            destination, source = tuple(map(lambda x: x.strip(), line.split("->")))
            destination = Path(destination)
            source = Path(source)

            if root in destination.parents:
                result.append(GCRoot(source=source, destination=destination))

    return result
