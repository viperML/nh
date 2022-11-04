pub mod commands;
pub mod interface;
pub mod nixos;

use fern::colors::Color;
use log::{debug, SetLoggerError};

use crate::interface::NHParser;

fn main() -> anyhow::Result<()> {
    setup_logging()?;
    debug!("Logging setup!");

    let args = <NHParser as clap::Parser>::parse();

    args.command.run();

    Ok(())
}

fn setup_logging() -> Result<(), SetLoggerError> {
    let loglevel = if cfg!(debug_assertions) {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    };

    let color_text = fern::colors::ColoredLevelConfig::new()
        .debug(Color::BrightBlack)
        .warn(Color::White)
        .error(Color::White);

    let color_symbol = fern::colors::ColoredLevelConfig::new()
        .info(Color::Green)
        .debug(Color::BrightBlack);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_symbol}>>>\x1B[0m {color_line}{message}\x1B[0m",
                color_symbol = format_args!(
                    "\x1B[{}m",
                    color_symbol.get_color(&record.level()).to_fg_str()
                ),
                color_line = format_args!(
                    "\x1B[{}m",
                    color_text.get_color(&record.level()).to_fg_str()
                ),
                message = message,
            ));
        })
        .level(loglevel)
        .chain(std::io::stdout())
        .apply()
}
