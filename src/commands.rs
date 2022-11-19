use log::{debug, info};

use crate::{interface::{self, NHCommand}, nixos::RunError};

pub trait NHRunnable {
    fn run(&self) -> anyhow::Result<()>;
}

impl NHRunnable for interface::NHCommand {
    fn run(&self) -> anyhow::Result<()> {
        match self {
            NHCommand::Os(os_args) => os_args.run(),
            NHCommand::Clean(clean_args) => clean_args.run(),
            s => todo!("Subcommand {s:?} not yet implemented!"),
        }
    }
}

pub fn run_command(cmd: &str, dry: bool, info: Option<&str>) -> Result<(), RunError> {
    if let Some(msg) = info {
        info!("{msg}");
    }

    debug!("{cmd}");

    if !dry {
        let mut argv = cmd.split(' ');
        let arg0 = argv.next().expect("Bad command");
        let mut child = subprocess::Exec::cmd(arg0)
            .args(&argv.collect::<Vec<_>>())
            .popen()?;

        let exit = child.wait()?;
        if !exit.success() {
            let msg: String = match exit {
                subprocess::ExitStatus::Exited(code) => code.to_string(),
                subprocess::ExitStatus::Signaled(code) => code.to_string(),
                _ => format!("Unknown error: {:?}", exit),
            };
            return Err(RunError::ExitError(msg));
        };
    }

    Ok(())
}
