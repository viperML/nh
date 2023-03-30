

use anyhow::{bail, Context};


use thiserror::Error;

use log::{debug, info, trace};



use crate::commands::{NHRunnable};
use crate::interface::OsRebuildType::{self, Boot, Switch, Test};
use crate::interface::{self, OsRebuildArgs};
use crate::*;

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

impl NHRunnable for interface::OsArgs {
    fn run(&self) -> anyhow::Result<()> {
        trace!("{:?}", self);

        match &self.action {
            Switch(args) | Boot(args) | Test(args) => args.rebuild(&self.action),
            s => bail!("Subcommand {:?} not yet implemented", s),
        }
    }
}

impl OsRebuildArgs {
    pub fn rebuild(&self, rebuild_type: &OsRebuildType) -> anyhow::Result<()> {
        if nix::unistd::Uid::effective().is_root() {
            bail!("Don't run nh os as root. I will call sudo internally as needed");
        }

        let hostname = match &self.hostname {
            Some(h) => h.to_owned(),
            None => hostname::get().context("Failed to get hostname")?,
        };

        let out_dir = tempfile::Builder::new().prefix("nh-os-").tempdir()?;
        let out_link = out_dir.path().join("result");
        let out_link_str = out_link.to_str().unwrap();
        debug!("out_dir: {:?}", out_dir);
        debug!("out_link {:?}", out_link);

        let flake_output = format!(
            "{}#nixosConfigurations.{:?}.config.system.build.toplevel",
            &self.flakeref, hostname
        );

        commands::BuildCommandBuilder::default()
            .flakeref(flake_output)
            .message("Building NixOS configuration")
            .extra_args(&["--out-link", out_link_str])
            .build()?
            .run()?;

        let current_specialisation = std::fs::read_to_string(SPEC_LOCATION).ok();

        let target_specialisation =
            current_specialisation.or_else(|| self.specialisation.to_owned());

        debug!("target_specialisation: {target_specialisation:?}");

        let target_profile = match &target_specialisation {
            None => out_link.to_owned(),
            Some(spec) => out_link.join("specialisation").join(spec),
        };

        target_profile.try_exists().context("Doesn't exist")?;

        commands::CommandBuilder::default()
            .args(&[
                "nvd",
                "diff",
                CURRENT_PROFILE,
                target_profile.to_str().unwrap(),
            ])
            .message("Comparing changes")
            .build()?
            .run()?;

        if self.dry {
            return Ok(());
        }

        if self.ask {
            info!("Apply the config?");
            let confirmation = dialoguer::Confirm::new().default(false).interact()?;

            if !confirmation {
                return Err(OsRebuildError::NoConfirm.into());
            }
        }

        commands::CommandBuilder::default()
            .args(&[
                "sudo",
                "nix-env",
                "--profile",
                SYSTEM_PROFILE,
                "--set",
                out_link_str,
            ])
            .build()?
            .run()?;

        if let Test(_) | Switch(_) = rebuild_type {
            // !! Use the target profile aka spec-namespaced
            let switch_to_configuration =
                target_profile.join("bin").join("switch-to-configuration");
            let switch_to_configuration = switch_to_configuration.to_str().unwrap();

            commands::CommandBuilder::default()
                .args(&["sudo", switch_to_configuration, "test"])
                .message("Activating configuration")
                .build()?
                .run()?;
        }

        if let Boot(_) | Switch(_) = rebuild_type {
            // !! Use the base profile aka no spec-namespace
            let switch_to_configuration = out_link.join("bin").join("switch-to-configuration");
            let switch_to_configuration = switch_to_configuration.to_str().unwrap();

            commands::CommandBuilder::default()
                .args(&["sudo", switch_to_configuration, "test"])
                .message("Adding configuration to bootloader")
                .build()?
                .run()?;
        }

        // Drop the out dir *only* when we are finished
        drop(out_dir);

        Ok(())
    }
}
