use ambassador::{delegatable_trait, Delegate};
use anstyle::Style;
use clap::{builder::Styles, Args, Parser, Subcommand};
use clean_path::Clean;
use color_eyre::Result;
use std::{ffi::OsString, num::ParseIntError, time::Duration};

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

fn make_style() -> Styles {
    Styles::plain().header(Style::new().bold()).literal(
        Style::new()
            .bold()
            .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
    )
}

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = None,
    styles=make_style(),
    propagate_version = false,
)]
/// nh is yet another nix helper
pub struct NHParser {
    #[arg(short, long, global = true)]
    /// Show debug logs
    pub verbose: bool,

    #[command(subcommand)]
    pub command: NHCommand,
}

#[delegatable_trait]
pub trait NHRunnable {
    fn run(&self) -> Result<()>;
}

#[derive(Subcommand, Debug, Delegate)]
#[delegate(NHRunnable)]
#[command(disable_help_subcommand = true)]
pub enum NHCommand {
    Os(OsArgs),
    Home(HomeArgs),
    #[command(hide = true)]
    Search(SearchArgs),
    Clean(CleanProxy),
    Completions(CompletionArgs),
}

#[derive(Args, Debug)]
#[clap(verbatim_doc_comment)]
/// NixOS functionality
///
/// Reimplementations of nixos-rebuild
pub struct OsArgs {
    #[command(subcommand)]
    pub action: OsRebuildType,
}

#[derive(Debug, Subcommand)]
pub enum OsRebuildType {
    /// Build and activate the new configuration, and make it the boot default
    Switch(OsRebuildArgs),
    /// Build the new configuration and make it the boot default
    Boot(OsRebuildArgs),
    /// Build and activate the new configuration
    Test(OsRebuildArgs),
    /// Show an overview of the system's info
    #[command(hide = true)]
    Info,
}

#[derive(Debug, Args)]
pub struct OsRebuildArgs {
    #[command(flatten)]
    pub common: CommonRebuildArgs,

    /// Output to choose from the flakeref. Hostname is used by default
    #[arg(long, short = 'H', global = true)]
    pub hostname: Option<OsString>,

    /// Name of the specialisation
    #[arg(long, short)]
    pub specialisation: Option<String>,

    /// Extra arguments passed to nix build
    #[arg(last = true)]
    pub extra_args: Vec<String>,
}

#[derive(Debug, Args)]
pub struct CommonRebuildArgs {
    /// Only print actions, without performing them
    #[arg(long, short = 'n')]
    pub dry: bool,

    /// Ask for confirmation
    #[arg(long, short)]
    pub ask: bool,

    /// Flake reference to build
    #[arg(env = "FLAKE", value_hint = clap::ValueHint::DirPath)]
    pub flakeref: FlakeRef,

    /// Use nix-output-monitor for the build process
    #[arg(
        long,
        env = "NH_NOM",
        value_parser(clap::builder::FalseyValueParser::new())
    )]
    pub nom: bool,
}

#[derive(Args, Debug)]
/// Search a package
pub struct SearchArgs {
    #[arg(long, short)]
    max_results: usize,
}

// Needed a struct to have multiple sub-subcommands
#[derive(Debug, Clone, Args, Delegate)]
#[delegate(NHRunnable)]
pub struct CleanProxy {
    #[clap(subcommand)]
    command: CleanMode,
}

#[derive(Debug, Clone, Subcommand)]
/// Enhanced nix cleanup
pub enum CleanMode {
    /// Elevate to root to clean all profiles and gcroots
    All(CleanArgs),
    /// Clean your user's profiles and gcroots
    User(CleanArgs),
    /// Print information about the store of the system
    #[clap(hide = true)]
    Info,
}

#[derive(Args, Clone, Debug)]
#[clap(verbatim_doc_comment)]
/// Enhanced nix cleanup
///
/// For --keep-since, see the documentation of humantime for possible formats: https://docs.rs/humantime/latest/humantime/fn.parse_duration.html
pub struct CleanArgs {
    #[arg(long, short, default_value = "1")]
    /// At least keep this number of generations
    pub keep: u32,

    #[arg(
        long,
        short = 'K',
        default_value="0s",
    )]
    /// At least keep gcroots and generations in this time range since now.
    pub keep_since: humantime::Duration,

    /// Only print actions, without performing them
    #[arg(long, short = 'n')]
    pub dry: bool,

    /// Ask for confimation
    #[arg(long, short)]
    pub ask: bool,

}

#[derive(Debug, Args)]
/// Home-manager functionality
pub struct HomeArgs {
    #[command(subcommand)]
    pub subcommand: HomeSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum HomeSubcommand {
    #[clap(verbatim_doc_comment)]
    /// Build and activate a home-manager configuration
    ///
    /// Will check the current $USER and $(hostname) to determine which output to build, unless -c is passed
    Switch(HomeRebuildArgs),

    /// Show an overview of the installation
    #[command(hide(true))]
    Info,
}

#[derive(Debug, Args)]
#[clap(verbatim_doc_comment)]
pub struct HomeRebuildArgs {
    #[command(flatten)]
    pub common: CommonRebuildArgs,

    /// Name of the flake homeConfigurations attribute, like username@hostname
    #[arg(long, short)]
    pub configuration: Option<String>,

    /// Extra arguments passed to nix build
    #[arg(last = true)]
    pub extra_args: Vec<String>,
}

#[derive(Debug, Parser)]
/// Generate shell completion files into stdout
pub struct CompletionArgs {
    /// Name of the shell
    #[arg(long, short)]
    pub shell: clap_complete::Shell,
}
