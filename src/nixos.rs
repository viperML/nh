use std::path::{PathBuf};

use clean_path::Clean;

use log::{debug, info};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::interface::{self, RebuildType};

const SYSTEM_PROFILE: &str = "/nix/var/nix/profiles/system";

#[derive(Debug)]
pub enum RunError {
    PopenError,
    ExitError,
    IoError,
    NoConfirm,
    SpecialisationError(String),
}

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
    info.map(|i| info!("{}", i));

    debug!("{cmd}");

    if !dry {
        let mut argv = cmd.split(' ');
        let arg0 = argv.next().expect("Bad command");
        let output = subprocess::Exec::cmd(arg0)
            .args(&argv.collect::<Vec<_>>())
            .capture()?;

        if !output.success() {
            return Err(RunError::ExitError);
        }
    };

    Ok(())
}

fn make_path_exists(elems: Vec<&str>) -> Option<String> {
    let p = PathBuf::from(elems.join("")).clean();

    match p.try_exists() {
        Err(_) => None,
        Ok(x) => {
            if x {
                p.to_str().map(String::from)
            } else {
                None
            }
        }
    }
}

impl interface::RebuildArgs {
    pub fn rebuild(&self, rebuild_type: interface::RebuildType) -> Result<(), RunError> {
        let hostname = hostname::get().expect("Failed to get hostname!");

        let flake_clean = self.flake.clean();

        let suffix_bytes = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<Vec<_>>();
        let suffix = String::from_utf8_lossy(&suffix_bytes);

        // let out_link = make_path(vec!["", &suffix]).unwrap();
        let out_link = format!("/tmp/nh/result-{}", &suffix);

        let cmd_build = vec![
            "nix",
            "build",
            "--out-link",
            &out_link,
            "--profile",
            SYSTEM_PROFILE,
            &format!(
                "{}#nixosConfigurations.{}.config.system.build.toplevel",
                &flake_clean.to_string_lossy(),
                &hostname.to_string_lossy()
            ),
        ]
        .join(" ");

        run_command(&cmd_build, self.dry, Some("Building"))?;

        let current_specialisation = get_specialisation();

        let target_specialisation = if self.specialisation.is_none() {
            &current_specialisation
        } else {
            &self.specialisation
        };

        let target_profile = if !self.dry {
            match target_specialisation {
                None => Ok(out_link.clone()),
                Some(spec) => {
                    let result = make_path_exists(vec![&out_link, spec]);
                    result.ok_or(RunError::SpecialisationError(spec.clone()))
                }
            }?
        } else {
            out_link.clone()
        };

        run_command(
            &vec!["nvd", "diff", SYSTEM_PROFILE, &target_profile].join(" "),
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

        match rebuild_type {
            RebuildType::Test | RebuildType::Switch => {
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

            RebuildType::Boot => {}
        }

        match rebuild_type {
            RebuildType::Boot | RebuildType::Switch => {
                let filename: &str = &format!("{}/bin/switch-to-configuration", out_link);
                let file = PathBuf::from(filename).clean();

                let cmd_activate: String = vec![file.to_str().unwrap(), "boot"].join(" ");
                run_command(&cmd_activate, self.dry, Some("Adding to bootloader"))?;
            }

            RebuildType::Test => {}
        }

        Ok(())
    }
}

fn get_specialisation() -> Option<String> {
    std::fs::read_to_string("/etc/specialisation").ok()
}
