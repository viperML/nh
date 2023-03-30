// Dont't use crate::
// We are getting called by build.rs

use clap::{Args, Parser, Subcommand};
use std::ffi::OsString;

#[derive(Debug, Clone, Default)]
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// Yet another nix helper
pub struct NHParser {
    #[arg(short, long, global = true)]
    /// Show debug logs
    pub verbose: bool,

    #[command(subcommand)]
    pub command: NHCommand,
}

#[derive(Subcommand, Debug)]
pub enum NHCommand {
    Os(OsArgs),
    Home(HomeArgs),
    #[cfg(debug_assertions)]
    Search(SearchArgs),
    Clean(CleanArgs),
    Completions(CompletionArgs),
}

#[derive(Args, Debug)]
/// NixOS related commands
pub struct OsArgs {
    #[command(subcommand)]
    pub action: OsRebuildType,
}

#[derive(Debug, Subcommand)]
pub enum OsRebuildType {
    /// Build, activate and set-for-boot
    Switch(OsRebuildArgs),
    /// Build and set-for-boot
    Boot(OsRebuildArgs),
    /// Build and activate
    Test(OsRebuildArgs),
    /// Show an overview of the system's info
    #[cfg(debug_assertions)]
    Info,
}

#[derive(Debug, Args)]
pub struct OsRebuildArgs {
    #[arg(long, short = 'n')]
    /// Only print actions to perform
    pub dry: bool,

    #[arg(long, short)]
    /// Confirm before performing the activation
    pub ask: bool,

    #[arg(env = "FLAKE", value_hint = clap::ValueHint::DirPath)]
    /// Flake reference that outputs a nixos system. Optionally add a #hostname
    pub flakeref: FlakeRef,

    #[arg(long, short = 'H', global = true)]
    /// Output to choose from the flakeref. Hostname is used by default
    pub hostname: Option<OsString>,

    #[arg(long, short)]
    /// Name of the specialisation
    pub specialisation: Option<String>,
}

#[derive(Args, Debug)]
/// Search a package
pub struct SearchArgs {
    #[arg(long, short)]
    max_results: usize,
    // #[arg(long, short)]
    // flake: String
}

#[derive(Args, Debug)]
#[clap(verbatim_doc_comment)]
/// Delete paths from the store
///
/// - Removes ALL inactive generations
/// - Calls nix-store --gc
///
pub struct CleanArgs {
    #[arg(long, short = 'n')]
    /// Only print actions to perform
    pub dry: bool,
}

#[derive(Debug, Args)]
/// Home-manager related commands
pub struct HomeArgs {
    #[command(subcommand)]
    pub subcommand: HomeSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum HomeSubcommand {
    /// Build and activate
    Switch(HomeRebuildArgs),
    /// Show an overview of the installation
    #[cfg(debug_assertions)]
    Info,
}

#[derive(Debug, Args)]
pub struct HomeRebuildArgs {
    #[arg(long, short = 'n')]
    /// Build the activation package, but don't run it
    pub dry: bool,

    #[arg(long, short)]
    /// Confirm before performing the activation
    pub ask: bool,

    #[arg(env = "FLAKE", value_hint = clap::ValueHint::DirPath)]
    /// Flake reference that outputs a nixos system. Optionally add a #hostname
    pub flakeref: FlakeRef,

    #[arg(long, short)]
    /// Name of the flake configuration: homeConfiguration.<name>
    pub configuration: Option<String>,

    /// Use nix-output-monitor for the build process
    #[arg(long, env = "NH_NOM")]
    pub nom: bool,

    #[arg(last = true)]
    /// Extra arguments passed to nix build
    pub extra_args: Vec<String>,
}

#[derive(Debug, Parser)]
/// Generate shell completion files into stdout
pub struct CompletionArgs {
    #[arg(long, short)]
    /// Name of the shell
    pub shell: clap_complete::Shell,
}
