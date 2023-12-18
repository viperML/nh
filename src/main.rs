mod clean;
mod commands;
mod completion;
mod home;
mod interface;
mod logging;
mod nixos;
mod search;

use crate::interface::NHParser;
use crate::interface::NHRunnable;
use color_eyre::Result;

fn main() -> Result<()> {
    logging::setup_logging()?;

    let args = <NHParser as clap::Parser>::parse();

    args.command.run()
}
