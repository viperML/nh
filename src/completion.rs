use crate::*;
use clap_complete::generate;
use color_eyre::Result;

#[async_trait::async_trait]
impl NHRunnable for interface::CompletionArgs {
    async fn run(&self) -> Result<()> {
        trace!("{:?}", self);
        let mut cmd = <NHParser as clap::CommandFactory>::command();
        generate(self.shell, &mut cmd, "nh", &mut std::io::stdout());
        Ok(())
    }
}
