use std::ops::Deref;

use color_eyre::eyre::{bail, Context};
use color_eyre::Result;

use log::{debug, info, trace};

use crate::interface::NHRunnable;
use crate::interface::OsRebuildType::{self, Boot, Switch, Test};
use crate::interface::{self, OsRebuildArgs};
use crate::*;

const SYSTEM_PROFILE: &str = "/nix/var/nix/profiles/system";
const CURRENT_PROFILE: &str = "/run/current-system";

const SPEC_LOCATION: &str = "/etc/specialisation";

impl NHRunnable for interface::OsArgs {
    fn run(&self) -> Result<()> {
        trace!("{:?}", self);

        match &self.action {
            Switch(args) | Boot(args) | Test(args) => args.rebuild(&self.action),
            s => bail!("Subcommand {:?} not yet implemented", s),
        }
    }
}

impl OsRebuildArgs {
    pub fn rebuild(&self, rebuild_type: &OsRebuildType) -> Result<()> {
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
            &self.common.flakeref.deref(),
            hostname
        );

        if self.common.update {
            commands::CommandBuilder::default()
                .args(&["nix", "flake", "update", &self.common.flakeref])
                .message("Updating flake")
                .build()?
                .exec()?;
        }

        commands::BuildCommandBuilder::default()
            .flakeref(flake_output)
            .message("Building NixOS configuration")
            .extra_args(&["--out-link", out_link_str])
            .extra_args(&self.extra_args)
            .nom(self.common.nom)
            .build()?
            .exec()?;

        let current_specialisation = std::fs::read_to_string(SPEC_LOCATION).ok();

        let target_specialisation = if self.no_specialisation {
            None
        } else {
            current_specialisation.or_else(|| self.specialisation.to_owned())
        };

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
            .exec()?;

        if self.common.dry {
            return Ok(());
        }

        if self.common.ask {
            info!("Apply the config?");
            let confirmation = dialoguer::Confirm::new().default(false).interact()?;

            if !confirmation {
                return Ok(());
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
            .exec()?;

        if let Test(_) | Switch(_) = rebuild_type {
            // !! Use the target profile aka spec-namespaced
            let switch_to_configuration =
                target_profile.join("bin").join("switch-to-configuration");
            let switch_to_configuration = switch_to_configuration.to_str().unwrap();

            commands::CommandBuilder::default()
                .args(&["sudo", switch_to_configuration, "test"])
                .message("Activating configuration")
                .build()?
                .exec()?;
        }

        if let Boot(_) | Switch(_) = rebuild_type {
            // !! Use the base profile aka no spec-namespace
            let switch_to_configuration = out_link.join("bin").join("switch-to-configuration");
            let switch_to_configuration = switch_to_configuration.to_str().unwrap();

            commands::CommandBuilder::default()
                .args(&["sudo", switch_to_configuration, "boot"])
                .message("Adding configuration to bootloader")
                .build()?
                .exec()?;
        }

        // Drop the out dir *only* when we are finished
        drop(out_dir);

        Ok(())
    }
}
