import subprocess
from pathlib import Path

import click

from .utils import nixfile


@click.group()
def cli() -> None:
    pass


@cli.command()
@click.argument("path")
def repl(path):
    """
    Start a Nix Repl and import files
    """

    repl_flake = Path(__file__).parent / "repl-flake.nix"

    try:
        my_nixfile = nixfile(path)
    except FileNotFoundError as e:
        raise FileNotFoundError from e

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

    pass
