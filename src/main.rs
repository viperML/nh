mod clean;
mod commands;
mod completion;
mod installable;
mod interface;
mod json;
mod logging;
mod nixos;
// mod search;
mod util;

use color_eyre::Result;
use tracing::debug;

const NH_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let mut do_warn = false;
    if let Ok(f) = std::env::var("FLAKE") {
        do_warn = true;
        if std::env::var("NH_FLAKE").is_err() {
            std::env::set_var("NH_FLAKE", f);
        }
    }

    let args = <crate::interface::Main as clap::Parser>::parse();
    crate::logging::setup_logging(args.verbose)?;
    tracing::debug!("{args:#?}");
    tracing::debug!(%NH_VERSION);

    if do_warn {
        tracing::warn!(
            "nh {NH_VERSION} now uses NH_FLAKE instead of FLAKE, please modify your configuration"
        );
    }

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
