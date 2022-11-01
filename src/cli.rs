use clap;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// Yet another nix helper
pub struct NHParser {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Args, Debug)]
/// Reimplementation of nixos-rebuild
pub struct Rebuild {
    #[arg(long, short)]
    /// Only print actions to perform
    dry: bool,

    #[arg(long, short)]
    /// Confirm before performing the activation
    ask: bool,

    #[arg(long, short)]
    /// Specialisation name
    specialisation: Option<String>,

    #[arg(env = "FLAKE")]
    /// Path to flake
    flake: std::path::PathBuf
}

#[derive(clap::Args, Debug)]
/// Search for a package
pub struct Search {
    #[arg(long, short)]
    max_results: usize,
    #[arg(long, short)]
    flake: String
}

#[derive(clap::Args, Debug)]
/// Clean-up garbage
pub struct Clean {}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    Switch(Rebuild),
    Boot(Rebuild),
    Test(Rebuild),
    Search(Search),
    Clean(Clean),
}
