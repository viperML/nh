import concurrent.futures
import json
import os
import platform
import random
import string
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Optional, Union

import click
from colorama import Fore as F

from .exceptions import CommandFailed, FlakeNotInitialized
from nh import deps

from types import SimpleNamespace


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


def run_cmd(cmd: str, tooltip: Optional[str], dry: bool) -> None:
    print()
    if tooltip:
        print(f">>> {F.GREEN}{tooltip}{F.RESET}")
    print(f"{F.LIGHTBLACK_EX}$ {cmd}{F.RESET}")

    if not dry:
        try:
            subprocess.run(cmd.split(" "), check=True)
        except KeyboardInterrupt:
            print(f">>> {F.RED}Operation cancelled by user!{F.RESET}")
            raise CommandFailed
        except subprocess.CalledProcessError:
            print(f">>> {F.RED}Operation cancelled, error during evaluation!{F.RESET}")
            raise CommandFailed


class NixProfile:
    SYSTEM_PROFILES = "/nix/var/nix/profiles/system"

    def __init__(self, path: Union[Path, str], default_spec: str, build: Optional[str]):
        self.path = Path(path)
        if build:
            cmd = f"nix build --profile {self.SYSTEM_PROFILES} --out-link {self.path} {build}"
            try:
                run_cmd(cmd=cmd, dry=False, tooltip="Building NixOS configuration")
            except CommandFailed:
                print(f">>> {F.RED}Build failure !{F.RESET}")
                exit(1)

        # Read all the subfolder names into self.specs
        self.specs = [
            x.name for x in (self.path / "specialisation").iterdir() if x.is_dir()
        ]
        self.default_spec = default_spec

        if self.default_spec:
            self.prefix = f"/specialisation/{self.default_spec}"
        else:
            self.prefix = ""

        if self.default_spec and self.default_spec not in self.specs:
            print(
                f">>> {F.RED}Specialisation {self.default_spec} not found in {self.path} !{F.RESET}"
                f">>> {F.RED}If you are using auto-spec detection, manually pass it with --specialisation{F.RESET}"
            )
            exit(1)

    def __str__(self) -> str:
        return str(self.path)

    def boot(self, dry: bool) -> None:
        cmd = f"{self.path}/bin/switch-to-configuration boot"
        run_cmd(
            cmd=cmd,
            dry=dry,
            tooltip="Adding profile to the bootloader",
        )

    def test(self, dry: bool) -> None:
        cmd = f"{self.path}{self.prefix}/bin/switch-to-configuration test"
        run_cmd(cmd=cmd, dry=dry, tooltip="Activating profile")

    # our self class as argument
    def diff(self, other) -> None:
        cmd = f"{deps.NVD} diff {self}{self.prefix} {other}{other.prefix}"
        run_cmd(cmd=cmd, dry=False, tooltip="Calculating transaction")


def current_spec() -> Optional[str]:
    try:
        with open(Path("/etc/specialisation"), "r") as f:
            return f.read()
    except FileNotFoundError:
        return None


def nixos_rebuild(ctx: click.core.Context) -> None:
    flake = str(NixFile(Path(ctx.params["flake"])))
    dry = ctx.params["dry_run"]
    hostname = platform.node()

    if ctx.params["specialisation"]:
        default_spec = ctx.params["specialisation"]
    else:
        default_spec = current_spec()

    profiles = SimpleNamespace(
        new=NixProfile(
            path=f'/tmp/nix-nh/{"".join(random.choice(string.ascii_letters) for i in range(17))}',
            default_spec=default_spec,
            build=f"{flake}#nixosConfigurations.{hostname}.config.system.build.toplevel",
        ),
        old=NixProfile(
            path="/run/current-system",
            default_spec=None,
            build=None,
        ),
    )

    profiles.old.diff(profiles.new)

    if (
        ctx.params["ask"]
        and not click.confirm(f">>> {F.YELLOW}Apply?{F.RESET}", default=True)
        and not ctx.params["dry_run"]
    ):
        exit(0)

    if ctx.command.name == "test" or ctx.command.name == "switch":
        profiles.new.test(dry)

    if ctx.command.name == "boot" or ctx.command.name == "switch":
        profiles.new.boot(dry)
