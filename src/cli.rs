use clap;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct NHParser {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Args, Debug)]
pub struct Rebuild {
    #[arg(long, short)]
    dry: bool,
    #[arg(long, short)]
    ask: bool,
    #[arg(long, short)]
    specialisation: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    Switch(Rebuild),
    Boot(Rebuild),
    Test(Rebuild)
}
