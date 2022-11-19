// Dont't use crate::
// We are getting called by build.rs

use std::ffi::OsString;
use clap::{Parser, Args, Subcommand};


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
    Search(SearchArgs),
    Clean(CleanArgs),
    Os(OsArgs),
}


#[derive(Args, Debug)]
/// NixOS related commands
pub struct OsArgs {
    #[command(subcommand)]
    pub action: RebuildType
}

#[derive(Debug, Subcommand)]
pub enum RebuildType {
    /// Build, activate and set-for-boot
    Switch(RebuildArgs),
    /// Build and set-for-boot
    Boot(RebuildArgs),
    /// Build and activate
    Test(RebuildArgs),
    /// Show an overview of the system's info
    Info,
}

#[derive(Debug, Args)]
pub struct RebuildArgs {
    #[arg(long, short)]
    /// Only print actions to perform
    pub dry: bool,

    #[arg(long, short)]
    /// Confirm before performing the activation
    pub ask: bool,

    #[arg(env = "FLAKE", value_hint = clap::ValueHint::DirPath)]
    /// Flake reference that outputs a nixos system. Optionally add a #hostname
    pub flakeref: FlakeRef,

    #[arg(long, short = 'H', global=true)]
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
/// Delete paths from the store
pub struct CleanArgs {}
