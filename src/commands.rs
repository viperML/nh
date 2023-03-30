use anyhow::bail;
use clap::builder;
use std::{
    fmt::Display,
    process::{self, Stdio},
};
use thiserror::Error;

use log::{debug, info, trace};
use rand::Rng;
use subprocess::{Exec, Redirection};

use crate::interface::{self, FlakeRef, NHCommand};

pub trait NHRunnable {
    fn run(&self) -> anyhow::Result<()>;
}

impl NHRunnable for interface::NHCommand {
    fn run(&self) -> anyhow::Result<()> {
        match self {
            NHCommand::Os(os_args) => os_args.run(),
            NHCommand::Clean(clean_args) => clean_args.run(),
            NHCommand::Home(home_args) => home_args.run(),
            NHCommand::Completions(args) => args.run(),
            s => bail!("Subcommand {s:?} not yet implemented!"),
        }
    }
}

pub fn run_command_capture(
    cmd: &Vec<&str>,
    message: Option<&str>,
) -> Result<String, subprocess::PopenError> {
    if let Some(m) = message {
        info!("{}", m);
    }

    debug!("{}", cmd.join(" "));

    let (head, tail) = cmd.split_at(1);
    let head = *head.first().unwrap();

    subprocess::Exec::cmd(head)
        .args(tail)
        .stdout(Redirection::Pipe)
        .capture()
        .map(|c| c.stdout_str().trim().to_owned())
}

#[derive(Debug, Error)]
pub enum RunError {
    #[error("Command exited with status {0}: {1}")]
    ExitError(String, String),
}

pub fn run_command<S>(cmd: &Vec<&str>, message: Option<S>, dry: bool) -> anyhow::Result<()>
where
    S: AsRef<str> + std::fmt::Display,
{
    if let Some(m) = message {
        info!("{}", m);
    }

    debug!("{}", cmd.join(" "));

    if !dry {
        let (head, tail) = cmd.split_at(1);
        let head = *head.first().unwrap();

        let exit = subprocess::Exec::cmd(head).args(tail).popen()?.wait()?;

        if !exit.success() {
            let code = match exit {
                subprocess::ExitStatus::Exited(code) => code.to_string(),
                subprocess::ExitStatus::Signaled(code) => code.to_string(),
                _ => format!("Unknown error: {:?}", exit),
            };

            // return Err(PopenError::LogicError("FIXME"));
            return Err(RunError::ExitError(code, cmd.join(" ")).into());
        };
    }

    Ok(())
}

pub fn mk_temp<P>(prefix: P) -> String
where
    P: AsRef<str> + Display,
{
    let suffix_bytes: Vec<_> = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(10)
        .collect();

    let suffix = std::str::from_utf8(&suffix_bytes).unwrap();

    format!("{}{}", &prefix, &suffix)
}

pub fn check_root() -> Result<(), anyhow::Error> {
    let euid = unsafe { libc::geteuid() };

    trace!("euid: {}", euid);

    if euid != 0 {
        bail!("This command requires root provileges!")
    } else {
        Ok(())
    }
}

#[derive(Debug, derive_builder::Builder, Default)]
#[builder(setter(into))]
pub struct Command {
    /// Arguments argv0..N
    args: Vec<String>,
    #[builder(default)]
    /// Whether to actually run the command or just log it
    dry: bool,
    #[builder(default)]
    /// Human-readable message regarding what the command does
    message: Option<String>,
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let [head, tail @ ..] = &*self.args else {
            bail!("Args was length 0");
        };

        let cmd = Exec::cmd(head)
            .args(tail)
            .stderr(Redirection::None)
            .stdout(Redirection::None);

        cmd.popen()?.wait()?;

        Ok(())
    }
}

#[derive(Debug, Default, derive_builder::Builder)]
#[builder(setter(into))]
pub struct BuildCommand {
    flakeref: String,
    #[builder(default)]
    extra_args: Vec<String>,
}

impl BuildCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let args = [
            "build",
            &self.flakeref,
            "--log-format",
            "internal-json",
            "--verbose",
        ];

        let cmd = {
            Exec::cmd("nix")
                .args(&args)
                .args(&self.extra_args)
                .stdout(Redirection::Pipe)
                .stderr(Redirection::Merge)
                | Exec::cmd("nom").args(&["--json"]).stdout(Redirection::None)
        }
        .popen()?;

        for mut proc in cmd {
            proc.wait()?;
        }

        Ok(())
    }
}
