import subprocess
from pathlib import Path
import platform

import click

from nh.utils import NixFile, find_nixfiles, cmd_print
from nh import deps


@click.group()
def cli() -> None:
    pass


@cli.command()
@click.argument("path", type=click.Path(exists=True), envvar="FLAKE")
def repl(path):
    """
    Load a flake into a nix repl

    PATH to any nix file or container folder. If nothing is passed, the environment variable $FLAKE will be used
    """

    repl_flake = Path(__file__).parent / "repl-flake.nix"

    my_nixfile = NixFile(path)

    if my_nixfile.is_flake:
        subprocess.run(["nix", "flake", "show", str(my_nixfile.path)])
        subprocess.run(
            [
                "nix",
                "repl",
                "--arg",
                "flakepath",
                str(my_nixfile.path),
                str(repl_flake),
            ]
        )
    else:
        print(f"You are trying to load ${my_nixfile.path}, which is not a flake")


@cli.command()
@click.argument("path", type=click.Path(exists=True), envvar="FLAKE")
@click.option(
    "-R",
    "--recursive",
    is_flag=True,
    help="If path is a directory, recurse nix file through it",
)
@click.option("-n", "--dry-run", is_flag=True, help="Print commands and exit")
def update(path, recursive, dry_run):
    """
    Update a flake or any nix file containing fetchFromGitHub

    PATH to any nix file or container folder. If nothing is passed, the environment variable $FLAKE will be used
    """

    my_path = Path(path).resolve()

    if recursive and my_path.is_dir():
        my_nixfiles = find_nixfiles(my_path)
    else:
        my_nixfiles = [NixFile(my_path)]

    for nf in my_nixfiles:
        if nf.is_flake:
            cmd = ["nix", "flake", "update", str((nf.path / "..").resolve())]
            cmd_print(cmd)
            if not dry_run:
                subprocess.run(cmd)

        elif nf.has_fetchFromGitHub:
            cmd = [deps.UNF, str(nf.path)]
            cmd_print(cmd)
            if not dry_run:
                subprocess.run(cmd)


@cli.command()
@click.argument("flake", type=click.Path(exists=True), envvar="FLAKE", required=False)
@click.option("-n", "--dry-run", is_flag=True, help="Print commands and exit")
def switch(flake, dry_run):
    """
    Call nixos-rebuild switch, and show the diff with nvd

    FLAKE path may be passed if the path is non standard (/etc/nixos).
    You can instead use the enviroment variable $FLAKE.
    """
    # TODO pass sudo as arg
    cmd = ["sudo", "nixos-rebuild", "switch"]

    profile_prev = Path("/nix/var/nix/profiles/system").resolve()

    if flake:
        fl = NixFile(Path(flake))
        cmd = [*cmd, "--flake", str(fl.path) + f"#{platform.node()}"]
    cmd_print(cmd)

    if not dry_run:
        subprocess.run(cmd)
        profile_new = Path("/nix/var/nix/profiles/system").resolve()
        cmd_nvd = [deps.NVD, "diff", str(profile_prev), str(profile_new)]
        subprocess.run(cmd_nvd)
