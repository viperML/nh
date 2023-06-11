mod clean;
mod commands;
mod completion;
mod home;
mod interface;
mod nixos;
mod search;

use color_eyre::Result;
use fern::colors::Color;
use log::{error, trace, SetLoggerError};

use crate::interface::NHParser;
use crate::interface::NHRunnable;

fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_location_section(false)
        .panic_section("consider reporting the bug at https://github.com/viperML/nh")
        .display_env_section(false)
        .install()?;

    let args = <NHParser as clap::Parser>::parse();

    setup_logging(args.verbose)?;

    args.command.run()
}

fn setup_logging(verbose: bool) -> Result<(), SetLoggerError> {
    let loglevel = if cfg!(debug_assertions) {
        log::LevelFilter::Trace
    } else if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    let color_text = fern::colors::ColoredLevelConfig::new()
        .debug(Color::BrightBlack)
        .error(Color::White)
        .trace(Color::BrightBlue);

    let color_symbol = fern::colors::ColoredLevelConfig::new()
        .debug(Color::BrightBlack)
        .error(Color::Red)
        .error(Color::Red)
        .info(Color::Green)
        .trace(Color::BrightBlue)
        .warn(Color::Yellow);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            let prefix = match record.level() {
                log::Level::Info | log::Level::Warn | log::Level::Error => "\n",
                _ => "",
            };
            out.finish(format_args!(
                "{prefix}{color_symbol}>\x1B[0m {color_line}{message}\x1B[0m",
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
