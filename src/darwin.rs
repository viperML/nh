use color_eyre::eyre::{bail, Context};
use tracing::{debug, info, warn};

use crate::commands;
use crate::commands::Command;
use crate::installable::Installable;
use crate::interface::{DarwinArgs, DarwinRebuildArgs, DarwinReplArgs, DarwinSubcommand};
use crate::nixos::toplevel_for;
use crate::update::update;
use crate::Result;

const SYSTEM_PROFILE: &str = "/nix/var/nix/profiles/system";
const CURRENT_PROFILE: &str = "/run/current-system";

impl DarwinArgs {
    pub fn run(self) -> Result<()> {
        use DarwinRebuildVariant::*;
        match self.subcommand {
            DarwinSubcommand::Switch(args) => args.rebuild(Switch),
            DarwinSubcommand::Build(args) => {
                if args.common.ask || args.common.dry {
                    warn!("`--ask` and `--dry` have no effect for `nh darwin build`");
                }
                args.rebuild(Build)
            }
            DarwinSubcommand::Repl(args) => args.run(),
        }
    }
}

enum DarwinRebuildVariant {
    Switch,
    Build,
}

fn get_hostname(hostname: Option<String>) -> Result<String> {
    match &hostname {
        Some(h) => Ok(h.to_owned()),
        None => {
            #[cfg(not(target_os = "macos"))]
            {
                Ok(hostname::get()
                    .context("Failed to get hostname")?
                    .to_str()
                    .unwrap()
                    .to_string())
            }
            #[cfg(target_os = "macos")]
            {
                use system_configuration::{
                    core_foundation::{base::TCFType, string::CFString},
                    sys::dynamic_store_copy_specific::SCDynamicStoreCopyLocalHostName,
                };

                let ptr = unsafe { SCDynamicStoreCopyLocalHostName(std::ptr::null()) };
                if ptr.is_null() {
                    bail!("Failed to get hostname");
                }
                let name = unsafe { CFString::wrap_under_get_rule(ptr) };

                Ok(name.to_string())
            }
        }
    }
}

impl DarwinRebuildArgs {
    fn rebuild(self, variant: DarwinRebuildVariant) -> Result<()> {
        use DarwinRebuildVariant::*;

        if nix::unistd::Uid::effective().is_root() {
            bail!("Don't run nh os as root. I will call sudo internally as needed");
        }

        if self.update_args.update {
            update(&self.common.installable, self.update_args.update_input)?;
        }

        let hostname = get_hostname(self.hostname)?;

        let out_path: Box<dyn crate::util::MaybeTempPath> = match self.common.out_link {
            Some(ref p) => Box::new(p.clone()),
            None => Box::new({
                let dir = tempfile::Builder::new().prefix("nh-os").tempdir()?;
                (dir.as_ref().join("result"), dir)
            }),
        };

        debug!(?out_path);

        let mut installable = self.common.installable.clone();
        if let Installable::Flake {
            ref mut attribute, ..
        } = installable
        {
            // If user explicitly selects some other attribute, don't push darwinConfigurations
            if attribute.is_empty() {
                attribute.push(String::from("darwinConfigurations"));
                attribute.push(hostname.clone());
            }
        }

        let toplevel = toplevel_for(hostname, installable);

        commands::Build::new(toplevel)
            .extra_arg("--out-link")
            .extra_arg(out_path.get_path())
            .extra_args(&self.extra_args)
            .message("Building Darwin configuration")
            .nom(!self.common.no_nom)
            .run()?;

        let target_profile = out_path.get_path().to_owned();

        target_profile.try_exists().context("Doesn't exist")?;

        Command::new("nvd")
            .arg("diff")
            .arg(CURRENT_PROFILE)
            .arg(&target_profile)
            .message("Comparing changes")
            .run()?;

        if self.common.ask && !self.common.dry && !matches!(variant, Build) {
            info!("Apply the config?");
            let confirmation = dialoguer::Confirm::new().default(false).interact()?;

            if !confirmation {
                bail!("User rejected the new config");
            }
        }

        if let Switch = variant {
            Command::new("nix")
                .args(["build", "--no-link", "--profile", SYSTEM_PROFILE])
                .arg(out_path.get_path())
                .elevate(true)
                .dry(self.common.dry)
                .run()?;

            let darwin_rebuild = out_path.get_path().join("sw/bin/darwin-rebuild");
            let activate_user = out_path.get_path().join("activate-user");

            // Determine if we need to elevate privileges
            let needs_elevation = !activate_user
                .try_exists()
                .context("Failed to check if activate-user file exists")?
                || std::fs::read_to_string(&activate_user)
                    .context("Failed to read activate-user file")?
                    .contains("# nix-darwin: deprecated");

            // Create and run the activation command with or without elevation
            Command::new(darwin_rebuild)
                .arg("activate")
                .message("Activating configuration")
                .elevate(needs_elevation)
                .dry(self.common.dry)
                .run()?;
        }

        // Make sure out_path is not accidentally dropped
        // https://docs.rs/tempfile/3.12.0/tempfile/index.html#early-drop-pitfall
        drop(out_path);

        Ok(())
    }
}

impl DarwinReplArgs {
    fn run(self) -> Result<()> {
        let mut target_installable = self.installable;

        if matches!(target_installable, Installable::Store { .. }) {
            bail!("Nix doesn't support nix store installables.");
        }

        let hostname = get_hostname(self.hostname)?;

        if let Installable::Flake {
            ref mut attribute, ..
        } = target_installable
        {
            if attribute.is_empty() {
                attribute.push(String::from("darwinConfigurations"));
                attribute.push(hostname);
            }
        }

        Command::new("nix")
            .arg("repl")
            .args(target_installable.to_args())
            .run()?;

        Ok(())
    }
}
