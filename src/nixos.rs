use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::Context;
use clean_path::Clean;
use thiserror::Error;

use log::trace;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::commands::{run_command, NHRunnable};
use crate::interface::OsRebuildType::{self, Boot, Info, Switch, Test};
use crate::interface::{self, OsRebuildArgs};

const SYSTEM_PROFILE: &str = "/nix/var/nix/profiles/system";
const CURRENT_PROFILE: &str = "/run/current-system";

const SPEC_LOCATION: &str = "/etc/specialisation";


#[derive(Debug, Error)]
pub enum OsRebuildError {
    #[error("No confirmation")]
    NoConfirm,
    #[error("Specialisation {0} does not exist")]
    SpecError(String),
}

/// Constructs a path by joining the elements, and checks if it exists.
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
    pub fn rebuild(&self, rebuild_type: &OsRebuildType) -> anyhow::Result<()> {
        let hostname = Box::new(match &self.hostname {
            Some(h) => h.into(),
            None => hostname::get().expect("FIXME"),
        });

        let suffix_bytes = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<Vec<_>>();

        let suffix = String::from_utf8(suffix_bytes).expect("Failed to get folder suffix");

        let out_link = format!("/tmp/nh/result-{}", &suffix);

        {
            let flake_output = format!(
                "{}#nixosConfigurations.{:?}.config.system.build.toplevel",
                &self.flakeref, hostname
            );

            let cmd_build = vec![
                "nix",
                "build",
                "--out-link",
                &out_link,
                "--profile",
                SYSTEM_PROFILE,
                &flake_output,
            ];

            run_command(&cmd_build, Some("Building configuration"), self.dry)
                .context("Failed during configuration build")?;
        }

        let current_specialisation = std::fs::read_to_string(SPEC_LOCATION).context(format!(
            "Failed to read specialisation from {}",
            SPEC_LOCATION
        ))?;

        let target_specialisation: Option<String> = if self.specialisation.is_none() {
            Some(current_specialisation)
        } else {
            self.specialisation.clone()
        };

        trace!("target_spec: {target_specialisation:?}");

        // if let Some(s) = &target_specialisation {};

        let target_profile = if !self.dry {
            match &target_specialisation {
                None => Ok(out_link.clone()),
                Some(spec) => {
                    let result = make_path_exists(vec![&out_link, "/specialisation/", spec]);
                    result.ok_or_else(|| OsRebuildError::SpecError(spec.to_owned()))
                }
            }?
        } else {
            out_link.clone()
        };

        run_command(
            &vec!["nvd", "diff", CURRENT_PROFILE, &target_profile],
            Some("Comparing changes"),
            self.dry,
        )?;

        if self.ask {
            let confirmation = dialoguer::Confirm::new()
                .with_prompt("Apply the config?")
                .default(false)
                .interact()?;

            if !confirmation {
                return Err(OsRebuildError::NoConfirm.into());
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

            let cmd_activate = vec![file.to_str().unwrap(), "test"];
            run_command(&cmd_activate, Some("Activating"), self.dry)?;
        }

        if let Boot(_) | Switch(_) = rebuild_type {
            let filename: &str = &format!("{}/bin/switch-to-configuration", out_link);
            let file = PathBuf::from(filename).clean();

            let cmd_activate = vec![file.to_str().unwrap(), "boot"];
            run_command(&cmd_activate, Some("Adding to bootloader"), self.dry)?;
        }

        Ok(())
    }
}
