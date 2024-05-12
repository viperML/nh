use std::env;
use std::ops::Deref;
use std::path::PathBuf;

use color_eyre::eyre::bail;
use color_eyre::Result;
use thiserror::Error;
use tracing::{debug, info, instrument};

use crate::*;
use crate::{
    interface::NHRunnable,
    interface::{FlakeRef, HomeArgs, HomeRebuildArgs, HomeSubcommand},
    util::{compare_semver, get_nix_version},
};

#[derive(Error, Debug)]
enum HomeRebuildError {
    #[error("Configuration \"{0}\" doesn't exist")]
    ConfigName(String),
}

impl NHRunnable for HomeArgs {
    fn run(&self) -> Result<()> {
        // self.subcommand
        match &self.subcommand {
            HomeSubcommand::Switch(args) | HomeSubcommand::Build(args) => {
                args.rebuild(&self.subcommand)
            }
            s => bail!("Subcommand {:?} not yet implemented", s),
        }
    }
}

impl HomeRebuildArgs {
    fn rebuild(&self, action: &HomeSubcommand) -> Result<()> {
        let out_dir = tempfile::Builder::new().prefix("nh-home-").tempdir()?;
        let out_link = out_dir.path().join("result");
        let out_link_str = out_link.to_str().unwrap();
        debug!("out_dir: {:?}", out_dir);
        debug!("out_link {:?}", out_link);

        let username = std::env::var("USER").expect("Couldn't get username");

        let hm_config_name = match &self.configuration {
            Some(name) => {
                if configuration_exists(&self.common.flakeref, name)? {
                    name.to_owned()
                } else {
                    return Err(HomeRebuildError::ConfigName(name.to_owned()).into());
                }
            }
            None => get_home_output(&self.common.flakeref, &username)?,
        };

        debug!("hm_config_name: {}", hm_config_name);

        let flakeref = format!(
            "{}#homeConfigurations.{}.config.home.activationPackage",
            &self.common.flakeref.deref(),
            hm_config_name
        );

        if self.common.update {
            // Get the Nix version
            let nix_version = get_nix_version().unwrap_or_else(|_| {
                panic!("Failed to get Nix version. Custom Nix fork?");
            });

            // Default interface for updating flake inputs
            let mut update_args = vec!["nix", "flake", "update"];

            // If user is on Nix 2.19.0 or above, --flake must be passed
            if let Ok(ordering) = compare_semver(&nix_version, "2.19.0") {
                if ordering == std::cmp::Ordering::Greater {
                    update_args.push("--flake");
                }
            }

            update_args.push(&self.common.flakeref);

            debug!("nix_version: {:?}", nix_version);
            debug!("update_args: {:?}", update_args);

            commands::CommandBuilder::default()
                .args(&update_args)
                .message("Updating flake")
                .build()?
                .exec()?;
        }

        commands::BuildCommandBuilder::default()
            .flakeref(&flakeref)
            .extra_args(["--out-link", out_link_str])
            .extra_args(&self.extra_args)
            .message("Building home configuration")
            .nom(!self.common.no_nom)
            .build()?
            .exec()?;

        let prev_generation: Option<PathBuf> = [
            PathBuf::from("/nix/var/nix/profiles/per-user")
                .join(username)
                .join("home-manager"),
            PathBuf::from(env::var("HOME").unwrap()).join(".local/state/nix/profiles/home-manager"),
        ]
        .into_iter()
        .fold(None, |res, next| {
            res.or_else(|| if next.exists() { Some(next) } else { None })
        });

        debug!("prev_generation: {:?}", prev_generation);

        // just do nothing for None case (fresh installs)
        if let Some(prev_gen) = prev_generation {
            commands::CommandBuilder::default()
                .args(self.common.diff_provider.split_ascii_whitespace())
                .args([(prev_gen.to_str().unwrap()), out_link_str])
                .message("Comparing changes")
                .build()?
                .exec()?;
        }

        if self.common.dry || matches!(action, HomeSubcommand::Build(_)) {
            return Ok(());
        }

        if self.common.ask {
            info!("Apply the config?");
            let confirmation = dialoguer::Confirm::new().default(false).interact()?;

            if !confirmation {
                return Ok(());
            }
        }

        if let Some(ext) = &self.backup_extension {
            info!("Using {} as the backup extension", ext);
            env::set_var("HOME_MANAGER_BACKUP_EXT", ext);
        }

        commands::CommandBuilder::default()
            .args([&format!("{}/activate", out_link_str)])
            .message("Activating configuration")
            .build()?
            .exec()?;

        // Drop the out dir *only* when we are finished
        drop(out_dir);

        Ok(())
    }
}

fn get_home_output<S: AsRef<str> + std::fmt::Display>(
    flakeref: &FlakeRef,
    username: S,
) -> Result<String> {
    // Replicate these heuristics
    // https://github.com/nix-community/home-manager/blob/433e8de330fd9c157b636f9ccea45e3eeaf69ad2/home-manager/home-manager#L110

    let hostname = hostname::get()
        .expect("Couldn't get hostname")
        .into_string()
        .unwrap();

    let username_hostname = format!("{}@{}", username, &hostname);

    if configuration_exists(flakeref, &username_hostname)? {
        Ok(username_hostname)
    } else if configuration_exists(flakeref, username.as_ref())? {
        Ok(username.to_string())
    } else {
        bail!(
            "Couldn't detect a home configuration for {}",
            username_hostname
        );
    }
}

#[instrument(ret, err, level = "debug")]
fn configuration_exists(flakeref: &FlakeRef, configuration: &str) -> Result<bool> {
    let output = format!("{}#homeConfigurations", flakeref.deref());
    let filter = format!(r#" x: x ? "{}" "#, configuration);

    let result = commands::CommandBuilder::default()
        .args(["nix", "eval", &output, "--apply", &filter])
        .build()?
        .exec_capture()?
        .unwrap();

    debug!(?result);

    match result.as_str().trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => bail!("Failed to parse nix-eval output: {}", result),
    }
}
