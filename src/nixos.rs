use color_eyre::eyre::{bail, Context};
use color_eyre::Result;
use tracing::{debug, info, warn};

use crate::commands;
use crate::commands::Command;
use crate::installable::{Installable, Installable2Context};
use crate::interface::OsSubcommand::{self};
use crate::interface::{self, OsRebuildArgs, OsReplArgs};

const SYSTEM_PROFILE: &str = "/nix/var/nix/profiles/system";
const CURRENT_PROFILE: &str = "/run/current-system";

const SPEC_LOCATION: &str = "/etc/specialisation";

impl<C: Installable2Context> interface::OsArgs<C> {
    pub fn run(self) -> Result<()> {
        use OsRebuildVariant::*;
        match self.subcommand {
            OsSubcommand::Boot(args) => args.rebuild(Boot),
            OsSubcommand::Test(args) => args.rebuild(Test),
            OsSubcommand::Switch(args) => args.rebuild(Switch),
            OsSubcommand::Build(args) => args.rebuild(Build),
            OsSubcommand::Repl(args) => args.run(),
        }
    }
}

#[derive(Debug)]
enum OsRebuildVariant {
    Build,
    Switch,
    Boot,
    Test,
}

impl OsRebuildArgs {
    fn rebuild(self, variant: OsRebuildVariant) -> Result<()> {
        use OsRebuildVariant::*;

        let elevate = if self.bypass_root_check {
            warn!("Bypassing root check, now running nix as root");
            false
        } else {
            if nix::unistd::Uid::effective().is_root() {
                bail!("Don't run nh os as root. I will call sudo internally as needed");
            }
            true
        };

        let hostname = match &self.hostname {
            Some(h) => h.to_owned(),
            None => hostname::get()
                .context("Failed to get hostname")?
                .to_str()
                .unwrap()
                .to_owned(),
        };

        let out_path: Box<dyn crate::util::MaybeTempPath> = match self.common.out_link {
            Some(ref p) => Box::new(p.clone()),
            None => Box::new({
                let dir = tempfile::Builder::new().prefix("nh-os").tempdir()?;
                (dir.as_ref().join("result"), dir)
            }),
        };

        debug!(?out_path);

        let toplevel = toplevel_for(hostname, self.common.installable.clone());

        commands::Build::new(toplevel)
            .extra_arg("--out-link")
            .extra_arg(out_path.get_path())
            .extra_args(&self.extra_args)
            .message("Building NixOS configuration")
            .nom(!self.common.no_nom)
            .run()?;

        let current_specialisation = std::fs::read_to_string(SPEC_LOCATION).ok();

        let target_specialisation = if self.no_specialisation {
            None
        } else {
            current_specialisation.or_else(|| self.specialisation.to_owned())
        };

        debug!("target_specialisation: {target_specialisation:?}");

        let target_profile = match &target_specialisation {
            None => out_path.get_path().to_owned(),
            Some(spec) => out_path.get_path().join("specialisation").join(spec),
        };

        target_profile.try_exists().context("Doesn't exist")?;

        Command::new("nvd")
            .arg("diff")
            .arg(CURRENT_PROFILE)
            .arg(&target_profile)
            .message("Comparing changes")
            .run()?;

        if self.common.dry || matches!(variant, Build) {
            return Ok(());
        }

        if self.common.ask {
            info!("Apply the config?");
            let confirmation = dialoguer::Confirm::new().default(false).interact()?;

            if !confirmation {
                bail!("User rejected the new config");
            }
        }

        if let Test | Switch = variant {
            // !! Use the target profile aka spec-namespaced
            let switch_to_configuration =
                target_profile.join("bin").join("switch-to-configuration");
            let switch_to_configuration = switch_to_configuration.to_str().unwrap();

            Command::new(switch_to_configuration)
                .arg("test")
                .message("Activating configuration")
                .elevate(elevate)
                .run()?;
        }

        if let Boot | Switch = variant {
            Command::new("nix")
                .elevate(elevate)
                .args(["build", "--profile", SYSTEM_PROFILE])
                .arg(out_path.get_path())
                .run()?;

            // !! Use the base profile aka no spec-namespace
            let switch_to_configuration = out_path
                .get_path()
                .join("bin")
                .join("switch-to-configuration");

            Command::new(switch_to_configuration)
                .arg("boot")
                .elevate(elevate)
                .message("Adding configuration to bootloader")
                .run()?;
        }

        // Make sure out_path is not accidentally dropped
        // https://docs.rs/tempfile/3.12.0/tempfile/index.html#early-drop-pitfall
        drop(out_path);

        Ok(())
    }
}

pub fn toplevel_for<S: AsRef<str>>(hostname: S, installable: Installable) -> Installable {
    let mut res = installable.clone();
    let hostname = hostname.as_ref().to_owned();

    let toplevel = ["config", "system", "build", "toplevel"]
        .into_iter()
        .map(String::from);

    match res {
        Installable::Flake {
            ref mut attribute, ..
        } => {
            // If user explicitely selects some other attribute, don't push nixosConfigurations
            if attribute.is_empty() {
                attribute.push(String::from("nixosConfigurations"));
                attribute.push(hostname);
            }
            attribute.extend(toplevel);
        }
        Installable::File {
            ref mut attribute, ..
        } => {
            attribute.extend(toplevel);
        }
        Installable::Expression {
            ref mut attribute, ..
        } => {
            attribute.extend(toplevel);
        }
        Installable::Store { .. } => {}
    }

    res
}

impl<C: Installable2Context> OsReplArgs<C> {
    fn run(self) -> Result<()> {
        todo!();

        Ok(())
    }
}
