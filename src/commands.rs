

use crate::interface::{self, NHCommand};

pub trait NHRunnable {
    fn run(&self) -> anyhow::Result<()>;
}

impl NHRunnable for interface::NHCommand {
    fn run(&self) -> anyhow::Result<()> {
        match self {
            NHCommand::Os(a) => a.run(),
            s => todo!("Subcommand {s:?} not yet implemented!"),
        }
    }
}
