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
        # Don't use the flake.nix, but the folder containing it
        my_nixfile.path = (my_nixfile.path / "..").resolve()

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


@cli.command(
    name="switch",
    context_settings=dict(
        ignore_unknown_options=True,
        allow_extra_args=True,
    ),
)
@click.argument("flake", type=click.Path(exists=True), envvar="FLAKE", required=False)
@click.option("-n", "--dry-run", is_flag=True, help="Print commands and exit.")
@click.pass_context
def nixos_rebuild_switch(ctx, flake, dry_run):
    """
    Wrapper around nixos-rebuild <switch> and nvd.

    FLAKE: path to the flake to use. Will use environment variable $FLAKE, if nothing is passed.

    Extra options will be forwarded to nixos-rebuild.
    """
    nixos_rebuild(ctx)


@cli.command(
    name="boot",
    context_settings=dict(
        ignore_unknown_options=True,
        allow_extra_args=True,
    ),
)
@click.argument("flake", type=click.Path(exists=True), envvar="FLAKE", required=False)
@click.option("-n", "--dry-run", is_flag=True, help="Print commands and exit.")
@click.pass_context
def nixos_rebuild_boot(ctx, flake, dry_run):
    """
    Wrapper around nixos-rebuild <boot> and nvd.

    FLAKE: path to the flake to use. Will use environment variable $FLAKE, if nothing is passed.

    Extra options will be forwarded to nixos-rebuild.
    """
    nixos_rebuild(ctx)


@cli.command(
    name="test",
    context_settings=dict(
        ignore_unknown_options=True,
        allow_extra_args=True,
    ),
)
@click.argument("flake", type=click.Path(exists=True), envvar="FLAKE", required=False)
@click.option("-n", "--dry-run", is_flag=True, help="Print commands and exit.")
@click.pass_context
def nixos_rebuild_test(ctx, flake, dry_run):
    """
    Wrapper around nixos-rebuild <test> and nvd.

    FLAKE: path to the flake to use. Will use environment variable $FLAKE, if nothing is passed.

    Extra options will be forwarded to nixos-rebuild.
    """
    nixos_rebuild(ctx)


def nixos_rebuild(ctx: click.core.Context):
    my_flake = NixFile(Path(ctx.params["flake"]))
    cmd = [
        "sudo",
        "nixos-rebuild",
        ctx.command.name,
        "--flake",
        str(my_flake.path) + f"#{platform.node()}",
    ]
    if ctx.args:
        cmd.append(" ".join(ctx.args))

    cmd_print(cmd)

    profile_prev = Path("/run/current-system").resolve()

    if not ctx.params["dry_run"]:
        subprocess.run(cmd)
        profile_new = Path("/run/current-system").resolve()
        cmd_nvd = [deps.NVD, "diff", str(profile_prev), str(profile_new)]
        subprocess.run(cmd_nvd)
