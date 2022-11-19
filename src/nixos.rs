use std::ffi::OsString;
use std::path::PathBuf;

use clean_path::Clean;

use log::{debug, info, trace};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::commands::NHRunnable;
use crate::interface::OsRebuildType::{self, Boot, Info, Switch, Test};
use crate::interface::{self, OsRebuildArgs};

// use crate::interface::{self, RebuildType};

const SYSTEM_PROFILE: &str = "/nix/var/nix/profiles/system";
const CURRENT_PROFILE: &str = "/run/current-system";

#[derive(Debug)]
pub enum RunError {
    PopenError,
    ExitError(String),
    IoError,
    NoConfirm,
    SpecialisationError(String),
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RunError {}

impl From<subprocess::PopenError> for RunError {
    fn from(_: subprocess::PopenError) -> Self {
        RunError::PopenError
    }
}

impl From<std::io::Error> for RunError {
    fn from(_: std::io::Error) -> Self {
        RunError::IoError
    }
}

fn run_command(cmd: &str, dry: bool, info: Option<&str>) -> Result<(), RunError> {
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

fn make_path_exists(elems: Vec<&str>) -> Option<String> {
    let p = PathBuf::from(elems.join("")).clean();
    trace!("checking {p:?}");

    if p.exists() {
        p.to_str().map(String::from)
    } else {
        None
    }
}

impl NHRunnable for interface::OsArgs {
    fn run(&self) -> anyhow::Result<()> {
        trace!("{:?}", self);

        match &self.action {
            Switch(args) | Boot(args) | Test(args) => {
                args.rebuild(&self.action)?;
            }
            Info => {
                todo!()
            }
        }
        Ok(())
    }
}

impl OsRebuildArgs {
    pub fn rebuild(&self, rebuild_type: &OsRebuildType) -> Result<(), RunError> {
        let hostname: Box<OsString> = match &self.hostname {
            Some(h) => Box::new(h.into()),
            None => {
                let h = hostname::get().expect("FIXME");
                Box::new(h)
            }
        };

        let suffix_bytes = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<Vec<_>>();

        let suffix = String::from_utf8_lossy(&suffix_bytes);

        let out_link = format!("/tmp/nh/result-{}", &suffix);

        let cmd_build = vec![
            "nix",
            "build",
            "--out-link",
            &out_link,
            "--profile",
            SYSTEM_PROFILE,
            &format!(
                "{}#nixosConfigurations.{:?}.config.system.build.toplevel",
                &self.flakeref, hostname
            ),
        ]
        .join(" ");

        run_command(&cmd_build, self.dry, Some("Building configuration"))?;

        let current_specialisation = get_specialisation();

        let target_specialisation = if self.specialisation.is_none() {
            &current_specialisation
        } else {
            &self.specialisation
        };

        trace!("target_spec: {target_specialisation:?}");

        let target_profile = if !self.dry {
            match target_specialisation {
                None => Ok(out_link.clone()),
                Some(spec) => {
                    let result = make_path_exists(vec![&out_link, "/specialisation/", spec]);
                    result.ok_or_else(|| RunError::SpecialisationError(spec.clone()))
                }
            }?
        } else {
            out_link.clone()
        };

        run_command(
            &vec!["nvd", "diff", CURRENT_PROFILE, &target_profile].join(" "),
            self.dry,
            Some("Comparing changes"),
        )?;

        if self.ask {
            let confirmation = dialoguer::Confirm::new()
                .with_prompt("Apply the config?")
                .default(false)
                .interact()?;

            if !confirmation {
                return Err(RunError::NoConfirm);
            }
        }

        if let Test(_) | Switch(_) = rebuild_type {
            let specialisation_prefix = match target_specialisation {
                None => "/".to_string(),
                Some(s) => format!("/specialisation/{}", s),
            };

            let filename: &str = &format!(
                "{}{}/bin/switch-to-configuration",
                out_link, specialisation_prefix
            );
            let file = PathBuf::from(filename).clean();

            let cmd_activate: String = vec![file.to_str().unwrap(), "test"].join(" ");
            run_command(&cmd_activate, self.dry, Some("Activating"))?;
        }

        if let Boot(_) | Switch(_) = rebuild_type {
            let filename: &str = &format!("{}/bin/switch-to-configuration", out_link);
            let file = PathBuf::from(filename).clean();

            let cmd_activate: String = vec![file.to_str().unwrap(), "boot"].join(" ");
            run_command(&cmd_activate, self.dry, Some("Adding to bootloader"))?;
        }

        Ok(())
    }
}

fn get_specialisation() -> Option<String> {
    std::fs::read_to_string("/etc/specialisation").ok()
}
