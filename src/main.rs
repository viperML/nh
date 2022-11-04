pub mod interface;
pub mod nixos;
pub mod commands;

use log::{debug, SetLoggerError};

use crate::interface::{NHParser};

fn main() -> anyhow::Result<()> {
    setup_logging()?;
    debug!("Logging setup!");

    let args = <NHParser as clap::Parser>::parse();

    args.command.run();

    Ok(())
}

fn setup_logging() -> Result<(), SetLoggerError> {
    let loglevel = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} >> {}",
                // record.target(),
                record.level(),
                message
            ))
        })
        .level(loglevel)
        // - and per-module overrides
        // .level_for("hyper", log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
}
