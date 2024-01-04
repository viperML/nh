use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::*;
use color_eyre::eyre::{bail, Context, ContextCompat};
use regex::Regex;
use tracing::{debug, info, instrument, trace, warn};
use uzers::os::unix::UserExt;

// Nix impl:
// https://github.com/NixOS/nix/blob/master/src/nix-collect-garbage/nix-collect-garbage.cc

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Generation {
    number: u32,
    last_modified: SystemTime,
    path: PathBuf,
}

type ToBeCleaned = bool;
// BTreeMap to automatically sort generations by id
type GenerationsTagged = BTreeMap<Generation, ToBeCleaned>;
type ProfilesTagged = HashMap<PathBuf, GenerationsTagged>;

impl NHRunnable for interface::CleanMode {
    fn run(&self) -> Result<()> {
        let uid = nix::unistd::Uid::effective();

        let mut profiles = Vec::new();

        let args = match self {
            interface::CleanMode::Profile(args) => {
                profiles.push(args.profile.clone());
                &args.common
            }
            interface::CleanMode::All(args) => {
                if !uid.is_root() {
                    crate::self_elevate();
                }

                profiles.extend(profiles_in_dir("/nix/var/nix/profiles")?);

                for read_dir in PathBuf::from("/nix/var/nix/profiles/per-user").read_dir()? {
                    let path = read_dir?.path();
                    profiles.extend(profiles_in_dir(path)?);
                }

                for user in unsafe { uzers::all_users() } {
                    profiles.extend(profiles_in_dir(
                        user.home_dir().join(".local/state/nix/profiles"),
                    )?);
                }

                args
            }
            interface::CleanMode::User(args) => {
                if uid.is_root() {
                    bail!("nh clean user: don't run me as root!");
                }

                let user = nix::unistd::User::from_uid(uid)?.unwrap();

                profiles.extend(profiles_in_dir(
                    &PathBuf::from(std::env::var("HOME")?).join(".local/state/nix/profiles"),
                )?);
                profiles.extend(profiles_in_dir(
                    &PathBuf::from("/nix/var/nix/profiles/per-user").join(user.name),
                )?);

                args
            }
        };

        // Use mutation to raise errors as they come
        let mut profiles_tagged = ProfilesTagged::new();
        for p in profiles {
            profiles_tagged.insert(
                p.clone(),
                cleanable_generations(&p, args.keep, args.keep_since)?,
            );
        }

        prompt_clean(profiles_tagged, args.ask, args.dry)?;

        Ok(())
    }
}

#[instrument(ret, err, level = "trace")]
fn profiles_in_dir<P: AsRef<Path> + std::fmt::Debug>(dir: P) -> Result<Vec<PathBuf>> {
    let mut res = Vec::new();

    if let Ok(read_dir) = dir.as_ref().read_dir() {
        for entry in read_dir {
            let path = entry?.path();

            if let Ok(dst) = path.read_link() {
                let name = dst
                    .file_name()
                    .wrap_err("Reading file_name")?
                    .to_string_lossy();

                let generation_regex = Regex::new(r"^(.*)-(\d+)-link$")?;

                if let Some(_) = generation_regex.captures(&name) {
                    res.push(path);
                }
            }
        }
    }

    Ok(res)
}

#[instrument(err, level = "debug")]
fn cleanable_generations(
    profile: &Path,
    keep: u32,
    keep_since: humantime::Duration,
) -> Result<GenerationsTagged> {
    let name = profile
        .file_name()
        .context("Checking profile's name")?
        .to_str()
        .unwrap();

    let generation_regex = Regex::new(&format!(r"^{name}-(\d+)-link"))?;

    let mut result = GenerationsTagged::new();

    for entry in profile
        .parent()
        .context("Reading profile's parent dir")?
        .read_dir()
        .context("Reading profile's generations")?
    {
        let path = entry?.path();
        let captures = generation_regex.captures(path.file_name().unwrap().to_str().unwrap());

        if let Some(caps) = captures {
            if let Some(number) = caps.get(1) {
                let last_modified = std::fs::symlink_metadata(&path)
                    .context("Checking symlink metadata")?
                    .modified()
                    .context("Reading modified time")?;

                result.insert(
                    Generation {
                        number: number.as_str().parse().unwrap(),
                        last_modified,
                        path: path.clone(),
                    },
                    true,
                );
            }
        }
    }

    let now = SystemTime::now();
    for (gen, tbr) in result.iter_mut() {
        match now.duration_since(gen.last_modified) {
            Err(err) => {
                warn!(?err, ?now, ?gen, "Failed to compare time!");
            }
            Ok(val) if val <= keep_since.into() => {
                *tbr = false;
            }
            Ok(_) => {}
        }
    }

    for (_, tbr) in result.iter_mut().rev().take(keep as _) {
        *tbr = false;
    }

    debug!("{:#?}", result);
    Ok(result)
}

fn prompt_clean(profiles: ProfilesTagged, ask: bool, dry: bool) -> Result<()> {
    use owo_colors::OwoColorize;

    for (profile, generations_tagged) in profiles.iter() {
        println!("{}", profile.to_string_lossy().blue().bold());
        for (gen, tbr) in generations_tagged.iter().rev() {
            if *tbr {
                println!("- {} {}", "DEL".red(), gen.path.to_string_lossy());
            } else {
                println!("- {} {}", "OK ".green(), gen.path.to_string_lossy());
            };
        }
        println!();
    }

    if !dry {
        if ask {
            info!("Confirm the cleanup plan?");
            if !dialoguer::Confirm::new().default(false).interact()? {
                return Ok(());
            }
        }

        for (_, generations_tagged) in profiles.iter() {
            for (gen, tbr) in generations_tagged.iter().rev() {
                if *tbr {
                    info!("Removing {}", gen.path.to_string_lossy());
                    if let Err(err) = std::fs::remove_file(&gen.path) {
                        warn!(?err, "Failed to remove");
                    }
                }
            }
        }
    }

    Ok(())
}
