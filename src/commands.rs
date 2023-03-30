use anyhow::bail;

use std::{
    ffi::{OsStr, OsString},
    fmt::Display,
};
use thiserror::Error;

use log::{debug, info};
use rand::Rng;
use subprocess::{Exec, Redirection};

use crate::interface::{self, NHCommand};

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


#[derive(Debug, derive_builder::Builder, Default)]
#[builder(derive(Debug), setter(into), default)]
pub struct Command {
    /// Whether to actually run the command or just log it
    dry: bool,
    /// Human-readable message regarding what the command does
    #[builder(setter(strip_option))]
    message: Option<String>,
    /// Aruments 0..N
    #[builder(setter(custom), default = "vec![]")]
    args: Vec<OsString>,
}

impl CommandBuilder {
    pub fn args(&mut self, input: &[impl AsRef<OsStr>]) -> &mut Self {
        if let Some(args) = &mut self.args {
            args.extend(input.iter().map(|elem| elem.as_ref().to_owned()));
            self
        } else {
            self.args = Some(Vec::new());
            self.args(input)
        }
    }
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

        if let Some(m) = &self.message {
            info!("{}", m);
        }
        debug!("{:?}", cmd);

        cmd.popen()?.wait()?;

        Ok(())
    }
}

#[derive(Debug, Default, derive_builder::Builder)]
#[builder(setter(into), default)]
pub struct BuildCommand {
    /// Human-readable message regarding what the command does
    #[builder(setter(strip_option))]
    message: Option<String>,
    // Flakeref to build
    flakeref: String,
    // Extra arguments passed to nix build
    #[builder(setter(custom))]
    extra_args: Vec<OsString>,
}

impl BuildCommandBuilder {
    pub fn extra_args(&mut self, input: &[impl AsRef<OsStr>]) -> &mut Self {
        if let Some(args) = &mut self.extra_args {
            args.extend(input.iter().map(|elem| elem.as_ref().to_owned()));
            self
        } else {
            self.extra_args = Some(Vec::new());
            self.extra_args(input)
        }
    }
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
        };

        if let Some(m) = &self.message {
            info!("{}", m);
        }
        debug!("{:?}", cmd);

        let cmd = cmd.popen()?;

        for mut proc in cmd {
            proc.wait()?;
        }

        Ok(())
    }
}
