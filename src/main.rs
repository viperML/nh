mod clean;
mod commands;
mod completion;
mod home;
mod interface;
mod json;
mod logging;
mod nixos;
mod search;
mod util;

use crate::interface::NHParser;
use crate::interface::NHRunnable;
use color_eyre::Result;
use tracing::debug;

const NH_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let args = <NHParser as clap::Parser>::parse();
    crate::logging::setup_logging(args.verbose)?;
    tracing::debug!(?args);

    args.command.run()
}

fn self_elevate() -> ! {
    use std::os::unix::process::CommandExt;

    let mut cmd = std::process::Command::new("sudo");
    cmd.args(std::env::args());
    debug!("{:?}", cmd);
    let err = cmd.exec();
    panic!("{}", err);
}
