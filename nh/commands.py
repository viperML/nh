import subprocess
from pathlib import Path

import click

from .utils import nixfile, find_nixfiles


@click.group()
def cli() -> None:
    pass


@cli.command()
@click.argument("path", type=click.Path(exists=True), envvar="FLAKE")
def repl(path):
    """
    Load a flake into a nix repl
    """

    repl_flake = Path(__file__).parent / "repl-flake.nix"

    my_nixfile = nixfile(path)

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
@click.option("-R", "--recursive", is_flag=True)
@click.option("-n", "--dry-run", is_flag=True)
def update(path, recursive, dry_run):
    """
    Update a flake or any nix file
    """

    my_path = Path(path).resolve()

    if recursive and my_path.is_dir():
        my_nixfiles = find_nixfiles(my_path)
    else:
        my_nixfiles = [nixfile(my_path)]

    for nf in my_nixfiles:
        if nf.is_flake:
            cmd = ["nix", "flake", "update", str(nf.path)]
            click.echo("$ " + " ".join(cmd))
            if not dry_run:
                subprocess.run(cmd)

        elif nf.has_fetchFromGitHub:
            cmd = ["update-nix-fetchgit", str(nf.path)]
            click.echo("$ " + " ".join(cmd))
            if not dry_run:
                subprocess.run(cmd)
