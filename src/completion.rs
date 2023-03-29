use crate::*;
use clap_complete::generate;

impl NHRunnable for interface::CompletionArgs {
    fn run(&self) -> anyhow::Result<()> {
        trace!("{:?}", self);
        let mut cmd = <NHParser as clap::CommandFactory>::command();
        generate(self.shell, &mut cmd, "nh", &mut std::io::stdout());
        Ok(())
    }
}
