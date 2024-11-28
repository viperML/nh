use std::env;
use std::path::PathBuf;

use color_eyre::eyre::bail;
use color_eyre::Result;
use tracing::{debug, info};

use crate::commands;
use crate::commands::Command;
use crate::installable::Installable;
use crate::interface::{self, HomeRebuildArgs, HomeReplArgs, HomeSubcommand};
use crate::update::update;

impl interface::HomeArgs {
    pub fn run(self) -> Result<()> {
        use HomeRebuildVariant::*;
        match self.subcommand {
            HomeSubcommand::Switch(args) => args.rebuild(Switch),
            HomeSubcommand::Build(args) => args.rebuild(Build),
            HomeSubcommand::Repl(args) => args.run(),
        }
    }
}

#[derive(Debug)]
enum HomeRebuildVariant {
    Build,
    Switch,
}

impl HomeRebuildArgs {
    fn rebuild(self, variant: HomeRebuildVariant) -> Result<()> {
        use HomeRebuildVariant::*;

        if self.update_args.update {
            update(&self.common.installable, self.update_args.update_input)?;
        }

        let out_path: Box<dyn crate::util::MaybeTempPath> = match self.common.out_link {
            Some(ref p) => Box::new(p.clone()),
            None => Box::new({
                let dir = tempfile::Builder::new().prefix("nh-os").tempdir()?;
                (dir.as_ref().join("result"), dir)
            }),
        };

        debug!(?out_path);

        let toplevel = toplevel_for(self.common.installable.clone(), true)?;

        commands::Build::new(toplevel)
            .extra_arg("--out-link")
            .extra_arg(out_path.get_path())
            .extra_args(&self.extra_args)
            .message("Building Home-Manager configuration")
            .nom(!self.common.no_nom)
            .run()?;

        let prev_generation: Option<PathBuf> = [
            PathBuf::from("/nix/var/nix/profiles/per-user")
                .join(env::var("USER").expect("Couldn't get username"))
                .join("home-manager"),
            PathBuf::from(env::var("HOME").expect("Couldn't get home directory"))
                .join(".local/state/nix/profiles/home-manager"),
        ]
        .into_iter()
        .take_while(|next| next.exists())
        .next();

        debug!(?prev_generation);

        if let Some(generation) = prev_generation {
            Command::new("nvd")
                .arg("diff")
                .arg(generation)
                .arg(out_path.get_path())
                .message("Comparing changes")
                .run()?;
        }

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

        if let Some(ext) = &self.backup_extension {
            info!("Using {} as the backup extension", ext);
            env::set_var("HOME_MANAGER_BACKUP_EXT", ext);
        }

        Command::new(out_path.get_path().join("activate"))
            .message("Activating configuration")
            .run()?;

        // Make sure out_path is not accidentally dropped
        // https://docs.rs/tempfile/3.12.0/tempfile/index.html#early-drop-pitfall
        drop(out_path);

        Ok(())
    }
}

fn toplevel_for(installable: Installable, push_drv: bool) -> Result<Installable> {
    let mut res = installable.clone();

    let toplevel = ["config", "home", "activationPackage"]
        .into_iter()
        .map(String::from);

    match res {
        Installable::Flake {
            ref reference,
            ref mut attribute,
        } => 'flake: {
            // If user explicitely selects some other attribute, don't push homeConfigurations
            if !attribute.is_empty() {
                break 'flake;
            }

            attribute.push(String::from("homeConfigurations"));

            // check for <user> and <user@hostname>
            let username = std::env::var("USER").expect("Couldn't get username");
            let hostname = hostname::get()
                .expect("Couldn't get hostname")
                .to_str()
                .unwrap()
                .to_string();

            let mut tried = vec![];

            for attr in [format!("{username}@{hostname}"), username.to_string()] {
                let func = format!(r#" x: x ? "{}" "#, attr);
                let res = commands::Command::new("nix")
                    .args(["eval", "--apply"])
                    .arg(func)
                    .args(
                        (Installable::Flake {
                            reference: reference.clone(),
                            attribute: attribute.clone(),
                        })
                        .to_args(),
                    )
                    .run_capture()
                    .expect("Checking home-manager output");

                tried.push({
                    let mut attribute = attribute.clone();
                    attribute.push(attr.clone());
                    attribute
                });

                match res.as_deref() {
                    Some("true") => {
                        attribute.push(attr.clone());
                        if push_drv {
                            attribute.extend(toplevel);
                        }
                        break 'flake;
                    }
                    _ => {
                        continue;
                    }
                }
            }

            let tried_str = tried
                .into_iter()
                .map(|a| {
                    let f = Installable::Flake {
                        reference: reference.clone(),
                        attribute: a,
                    };
                    f.to_args().join(" ")
                })
                .collect::<Vec<_>>()
                .join(", ");

            bail!("Couldn't find home-manager configuration, tried {tried_str}");
        }
        Installable::File {
            ref mut attribute, ..
        } => {
            if push_drv {
                attribute.extend(toplevel);
            }
        }
        Installable::Expression {
            ref mut attribute, ..
        } => {
            if push_drv {
                attribute.extend(toplevel);
            }
        }
        Installable::Store { .. } => {}
    }

    Ok(res)
}

impl HomeReplArgs {
    fn run(self) -> Result<()> {
        let toplevel = toplevel_for(self.installable, false)?;

        Command::new("nix")
            .arg("repl")
            .args(toplevel.to_args())
            .run()?;

        Ok(())
    }
}
