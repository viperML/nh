// Dont't use crate::
// We are getting called by build.rs

use std::ffi::OsString;




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
    /// Name of the specialisation
    pub specialisation: Option<String>,

    #[arg(env = "FLAKE", value_hint = clap::ValueHint::DirPath)]
    /// Flake reference that outputs a nixos system. Optionally add a #hostname
    pub flakeref: FlakeRef,

    #[arg(long, short = 'H')]
    /// Output to choose from the flakeref. Hostname is used by default
    pub hostname: Option<OsString>
}

#[derive(Debug, Clone)]
pub struct FlakeRef(String);

impl From<&str> for FlakeRef {
    fn from(s: &str) -> Self {
        FlakeRef(s.to_string())
    }
}

impl std::fmt::Display for FlakeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug)]
pub enum RebuildType {
    Switch,
    Boot,
    Test,
}


#[derive(clap::Args, Debug)]
/// Search a package
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
