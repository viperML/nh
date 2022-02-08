import subprocess
import pathlib

import click

from .utils import find_flake


@click.group()
def cli() -> None:
    pass


@cli.command()
@click.argument("path")
def repl(path):
    """
    Start a Nix Repl and import files
    """
    try:
        flake_path = find_flake(path).parent
        repl = pathlib.Path(__file__).parent / "repl.nix"
        subprocess.run([
            "nix",
            "repl",
            "--arg", "flakePath", str(flake_path),
            str(repl)
        ])

    except FileNotFoundError as e:
        raise FileNotFoundError from e

    pass
