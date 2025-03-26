use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::eyre::{bail, Context};
use color_eyre::eyre::{eyre, Result};
use tracing::{debug, info, warn};

use crate::commands;
use crate::commands::Command;
use crate::generations;
use crate::installable::Installable;
use crate::interface::OsSubcommand::{self};
use crate::interface::{self, OsGenerationsArgs, OsBuildVmArgs, OsRebuildArgs, OsReplArgs};
use crate::update::update;

const SYSTEM_PROFILE: &str = "/nix/var/nix/profiles/system";
const CURRENT_PROFILE: &str = "/run/current-system";

const SPEC_LOCATION: &str = "/etc/specialisation";

impl interface::OsArgs {
    pub fn run(self) -> Result<()> {
        use OsRebuildVariant::*;
        match self.subcommand {
            OsSubcommand::Boot(args) => args.rebuild(Boot, None),
            OsSubcommand::Test(args) => args.rebuild(Test, None),
            OsSubcommand::Switch(args) => args.rebuild(Switch, None),
            OsSubcommand::Build(args) => {
                if args.common.ask || args.common.dry {
                    warn!("`--ask` and `--dry` have no effect for `nh os build`");
                }
                args.rebuild(Build, None)
            }
            OsSubcommand::BuildVm(args) => args.build(BuildVm),
            OsSubcommand::Repl(args) => args.run(),
            OsSubcommand::Info(args) => args.info(),
        }
    }
}

#[derive(Debug)]
enum OsRebuildVariant {
    Build,
    Switch,
    Boot,
    Test,
    BuildVm,
}

impl OsBuildVmArgs {
    fn build(self, variant: OsRebuildVariant) -> Result<()> {
        use OsRebuildVariant::*;

        let final_attr = Some(get_final_attr(true, self.with_bootloader));

        self.common.rebuild(variant, final_attr)
    }
}

impl OsRebuildArgs {
    // final_attr is the attribute of config.system.build.X to evaluate.
    fn rebuild(self, variant: OsRebuildVariant, final_attr: Option<String>) -> Result<()> {
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

        if self.update_args.update {
            update(&self.common.installable, self.update_args.update_input)?;
        }

        let system_hostname = hostname::get()
            .context("Failed to get hostname")?
            .to_str()
            .unwrap()
            .to_owned();

        let target_hostname = match &self.hostname {
            Some(h) => h.to_owned(),
            None => {
                tracing::warn!("Guessing system is {system_hostname} for a VM image. If this isn't intended, use --hostname to change.");
                system_hostname.clone()
            }
        };

        let out_path: Box<dyn crate::util::MaybeTempPath> = match self.common.out_link {
            Some(ref p) => Box::new(p.clone()),
            None => Box::new({
                let dir = tempfile::Builder::new().prefix("nh-os").tempdir()?;
                (dir.as_ref().join("result"), dir)
            }),
        };

        debug!(?out_path);

        let toplevel = toplevel_for(
            &target_hostname,
            self.common.installable.clone(),
            final_attr.unwrap_or(String::from("toplevel")),
        );

        let message = match variant {
            BuildVm => "Building NixOS VM image",
            _ => "Building NixOS configuration",
        };

        commands::Build::new(toplevel)
            .extra_arg("--out-link")
            .extra_arg(out_path.get_path())
            .extra_args(&self.extra_args)
            .message(message)
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

        if target_hostname == system_hostname {
            Command::new("nvd")
                .arg("diff")
                .arg(CURRENT_PROFILE)
                .arg(&target_profile)
                .message("Comparing changes")
                .run()?;
        } else {
            debug!("Not running nvd as the target hostname is different from the system hostname.")
        }

        if self.common.dry || matches!(variant, Build | BuildVm) {
            if self.common.ask {
                warn!("--ask has no effect as dry run was requested");
            }
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
                .args(["build", "--no-link", "--profile", SYSTEM_PROFILE])
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

pub fn get_final_attr(build_vm: bool, with_bootloader: bool) -> String {
    let attr = if build_vm && with_bootloader {
        "vmWithBootLoader"
    } else if build_vm {
        "vm"
    } else {
        "toplevel"
    };
    String::from(attr)
}

pub fn toplevel_for<S: AsRef<str>>(
    hostname: S,
    installable: Installable,
    final_attr: String,
) -> Installable {
    let mut res = installable.clone();
    let hostname = hostname.as_ref().to_owned();

    let toplevel = ["config", "system", "build", &final_attr]
        .into_iter()
        .map(String::from);

    match res {
        Installable::Flake {
            ref mut attribute, ..
        } => {
            // If user explicitly selects some other attribute, don't push nixosConfigurations
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

impl OsReplArgs {
    fn run(self) -> Result<()> {
        let mut target_installable = self.installable;

        if matches!(target_installable, Installable::Store { .. }) {
            bail!("Nix doesn't support nix store installables.");
        }

        let hostname = self
            .hostname
            .unwrap_or_else(|| hostname::get().unwrap().to_str().unwrap().to_string());

        if let Installable::Flake {
            ref mut attribute, ..
        } = target_installable
        {
            if attribute.is_empty() {
                attribute.push(String::from("nixosConfigurations"));
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

impl OsGenerationsArgs {
    fn info(&self) -> Result<()> {
        let profile = match self.profile {
            Some(ref p) => PathBuf::from(p),
            None => bail!("Profile path is required"),
        };

        if !profile.is_symlink() {
            return Err(eyre!(
                "No profile `{:?}` found",
                profile.file_name().unwrap_or_default()
            ));
        }

        let profile_dir = profile.parent().unwrap_or_else(|| Path::new("."));

        let generations: Vec<_> = fs::read_dir(profile_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path
                        .file_name()?
                        .to_str()?
                        .starts_with(profile.file_name()?.to_str()?)
                    {
                        Some(path)
                    } else {
                        None
                    }
                })
            })
            .collect();

        let descriptions: Vec<generations::GenerationInfo> = generations
            .iter()
            .filter_map(|gen_dir| generations::describe(gen_dir, &profile))
            .collect();

        generations::print_info(descriptions);

        Ok(())
    }
}
