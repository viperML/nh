use owo_colors::OwoColorize;
use tracing::Event;
use tracing::Level;
use tracing::Subscriber;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::Format;
use tracing_subscriber::fmt::FormatEvent;
use tracing_subscriber::fmt::FormatFields;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::EnvFilter;
use crate::*;

struct InfoFormatter;

impl<S, N> FormatEvent<S, N> for InfoFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: fmt::format::Writer,
        event: &Event,
    ) -> std::fmt::Result {
        // Based on https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/trait.FormatEvent.html#examples
        // Without the unused parts
        write!(writer, "{} ", ">".green())?;
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)?;
        Ok(())
    }
}

pub(crate) fn setup_logging() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_location_section(false)
        .panic_section("Please report the bug at https://github.com/viperML/nh/issues")
        .display_env_section(false)
        .install()?;

    let layer_debug = fmt::layer()
        .with_writer(std::io::stderr)
        .without_time()
        .compact()
        .with_line_number(true)
        .with_filter(EnvFilter::from_default_env())
        .with_filter(filter_fn(|meta| *meta.level() != Level::INFO));

    let layer_info = fmt::layer()
        .with_writer(std::io::stderr)
        .without_time()
        .with_target(false)
        .with_level(false)
        .event_format(InfoFormatter)
        .with_filter(filter_fn(|meta| *meta.level() == Level::INFO));

    tracing_subscriber::registry()
        .with(layer_debug)
        .with(layer_info)
        .init();

    tracing::trace!("Logging setup!");

    Ok(())
}