// Dont't use crate::
// We are getting called by build.rs




#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// Yet another nix helper
pub struct NHParser {
    #[arg(short)]
    // Show debug logs
    pub verbose: bool,

    #[command(subcommand)]
    pub command: NHCommand,
}

#[derive(clap::Args, Debug)]
/// Reimplementation of nixos-rebuild
pub struct RebuildArgs {
    #[arg(long, short)]
    /// Only print actions to perform
    pub dry: bool,

    #[arg(long, short)]
    /// Confirm before performing the activation
    pub ask: bool,

    #[arg(long, short)]
    /// Specialisation name
    pub specialisation: Option<String>,

    #[arg(env = "FLAKE")]
    /// Path to flake
    pub flake: std::path::PathBuf
}

#[derive(Debug)]
pub enum RebuildType {
    Switch,
    Boot,
    Test,
}

#[derive(clap::Args, Debug)]
/// Search for a package
pub struct SearchArgs {
    #[arg(long, short)]
    max_results: usize,
    #[arg(long, short)]
    flake: String
}

#[derive(clap::Args, Debug)]
/// Clean-up garbage
pub struct CleanArgs {}

#[derive(clap::Subcommand, Debug)]
pub enum NHCommand {
    Switch(RebuildArgs),
    Boot(RebuildArgs),
    Test(RebuildArgs),
    Search(SearchArgs),
    Clean(CleanArgs),
}
