use crate::*;
use clap_complete::generate;
use color_eyre::Result;
use tracing::trace;

impl NHRunnable for interface::CompletionArgs {
    fn run(&self) -> Result<()> {
        trace!("{:?}", self);
        let mut cmd = <NHParser as clap::CommandFactory>::command();
        generate(self.shell, &mut cmd, "nh", &mut std::io::stdout());
        Ok(())
    }
}
