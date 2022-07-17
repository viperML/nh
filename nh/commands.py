import json
import subprocess
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime
from functools import partial
from pathlib import Path

import click
import dateparser
from diskcache import Cache
from pyfzf.pyfzf import FzfPrompt
from xdg import xdg_cache_home

from nh import __version__, deps
from nh.utils import SearchResult, find_gcroots, nixos_rebuild


@click.group()
@click.version_option(__version__)
def cli() -> None:
    pass


@cli.command()
@click.argument(
    "path",
    type=click.Path(exists=True),
    envvar="FLAKE",
)
def repl(path):
    """
    Load a flake into a nix repl

    PATH to any nix file or container folder. If nothing is passed, the environment variable $FLAKE will be used
    """

    print("nh repl is deprecated. You can use `nix repl --file $FLAKE`")


@cli.command(
    context_settings=dict(
        ignore_unknown_options=True,
        allow_extra_args=True,
    ),
)
@click.argument("flake", type=click.Path(exists=True), envvar="FLAKE", required=False)
@click.option("-n", "--dry-run", is_flag=True, help="Print commands and exit.")
@click.option(
    "-S",
    "--no-auto-specialisation",
    is_flag=True,
    help="Disable automatic specialisation detection by reading /run/current-system-configuration-name",
)
@click.option("-s", "--specialisation", help="Name of the specialisation to use")
@click.option(
    "-a",
    "--ask",
    is_flag=True,
    help="Display the transaction and ask for confirmation.",
)
@click.pass_context
def switch(ctx, **kwargs):
    """
    Reimplementation of nixos-rebuild switch.

    Integrated with nvd, to show installed, removed and updated packages.
    """
    nixos_rebuild(ctx)


@cli.command(
    context_settings=dict(
        ignore_unknown_options=True,
        allow_extra_args=True,
    ),
)
@click.argument("flake", type=click.Path(exists=True), envvar="FLAKE", required=False)
@click.option("-n", "--dry-run", is_flag=True, help="Print commands and exit.")
@click.option(
    "-S",
    "--no-auto-specialisation",
    is_flag=True,
    help="Disable automatic specialisation detection by reading /run/current-system-configuration-name",
)
@click.option("-s", "--specialisation", help="Name of the specialisation to use")
@click.option(
    "-a",
    "--ask",
    is_flag=True,
    help="Display the transaction and ask for confirmation.",
)
@click.pass_context
def boot(ctx, **kwargs):
    """
    Reimplementation of nixos-rebuild boot.

    Integrated with nvd, to show installed, removed and updated packages.
    """
    nixos_rebuild(ctx)


@cli.command(
    context_settings=dict(
        ignore_unknown_options=True,
        allow_extra_args=True,
    ),
)
@click.argument(
    "flake", type=click.Path(exists=True, dir_okay=True), envvar="FLAKE", required=False
)
@click.option("-n", "--dry-run", is_flag=True, help="Print commands and exit.")
@click.option(
    "-S",
    "--no-auto-specialisation",
    is_flag=True,
    help="Disable automatic specialisation detection by reading /run/current-system-configuration-name",
)
@click.option("-s", "--specialisation", help="Name of the specialisation to use")
@click.option(
    "-a",
    "--ask",
    is_flag=True,
    help="Display the transaction and ask for confirmation.",
)
@click.pass_context
def test(ctx, **kwargs):
    """
    Reimplementation of nixos-rebuild test.

    Integrated with nvd, to show installed, removed and updated packages.
    """
    nixos_rebuild(ctx)


@click.option(
    "--flake",
    type=str,
    default="nixpkgs",
    show_default=True,
    required=False,
    help="""Flake to search in.""",
)
@click.option(
    "--max-results",
    type=int,
    default=10,
    show_default=True,
    required=False,
    help="""Maximum number of results with non-interactive search.
    May impact performance.""",
)
@click.argument("query", type=str, default=None, required=False)
@cli.command()
def search(flake, query, max_results):
    """
    Super fast search for packages.

    The first run will evaluate the flake and save a persistent snapshot
    to disk.

    QUERY can be left empty to get a interactive search with fzf.
    """
    fzf = FzfPrompt(deps.FZF)

    try:
        search_cache = Cache(directory=str(xdg_cache_home() / "nix-nh"))
        pkgs = search_cache.get(f"pkgs-{flake}")
        assert pkgs
    except AssertionError:
        pkgs_json = json.loads(
            subprocess.check_output(["nix", "search", "--json", flake]).decode()
        )
        pkgs = set()
        for p in pkgs_json:
            pkgs.add(f"{pkgs_json[p]['pname']}")
        # Free memory maybe?
        del pkgs_json
        search_cache.set(f"pkgs-{flake}", pkgs, expire=259200)

    fzf_options = "--height=20%"
    if query:
        fzf_options += f" --filter='{query}'"
    responses = fzf.prompt(pkgs, fzf_options=fzf_options)

    responses = responses[:max_results]
    responses.reverse()

    with ThreadPoolExecutor() as executor:
        results = executor.map(partial(SearchResult, flake=flake), responses)

    for r in results:
        print()
        r.print()

    print()


@cli.command(name="gcr-clean")
@click.option(
    "--age",
    type=str,
    default="",
    show_default=True,
    required=False,
    help="""Any gcroot created at a time older will be selected for removal.
            Accepts human readable values (e.g. '7 days ago').""",
)
@click.option("-n", "--dry-run", is_flag=True, help="Don't actually remove anything.")
@click.option(
    "--root",
    type=click.Path(exists=True, dir_okay=True),
    default=Path.home(),
    required=False,
    help="Root directory to scan from. Default: user's home dir.",
)
def gcr_clean(age, dry_run, root):
    """
    Find gcroots from a root directory, and delete them.
    A garbage collect root is a symlink from the store into a normal folder,
    and registered for gcroots, such that the dependencies of it won't be cleaned
    until you remove it (e.g build artifacts).
    """
    roots = find_gcroots(root)

    if age:
        max_age = dateparser.parse(age)
    else:
        max_age = datetime.now()

    for r in roots:
        if r.registration_time < max_age:
            print(f"Removing {r.destination}")
            if not dry_run:
                r.remove()
