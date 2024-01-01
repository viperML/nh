use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::SystemTime,
};

use color_eyre::eyre::{Context, ContextCompat};
use once_cell::sync::Lazy;
use regex::Regex;
use tracing::{debug, info, instrument, trace, warn};

use crate::*;

// Reference: https://github.com/NixOS/nix/blob/master/src/nix-collect-garbage/nix-collect-garbage.cc

impl NHRunnable for interface::CleanMode {
    fn run(&self) -> Result<()> {
        let uid = nix::unistd::Uid::effective();

        match self {
            interface::CleanMode::Profile(args) => {
                // cleanable_generations(args., keep, keep_size)
                let res =
                    cleanable_generations(&args.profile, args.common.keep, args.common.keep_since)?;
                let mut h = HashMap::new();
                h.insert(args.profile.clone(), res);
                prompt_clean(h, args.common.ask, args.common.dry)?;
            }
            interface::CleanMode::All(args) => todo!(),
            interface::CleanMode::User(args) => todo!(),
        }

        Ok(())
    }
}

type ToBeCleaned = bool;

#[instrument(err, level = "debug")]
fn cleanable_generations(
    profile: &Path,
    keep: u32,
    keep_since: humantime::Duration,
) -> Result<Vec<(Generation, ToBeCleaned)>> {
    let name = profile
        .file_name()
        .context("Checking profile's name")?
        .to_str()
        .unwrap();

    let generation_regex = Regex::new(&format!(r"{name}-(\d+)-link"))?;

    let mut generations = Vec::new();

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

                generations.push((
                    Generation {
                        number: number.as_str().parse().unwrap(),
                        last_modified,
                        path: path.clone(),
                    },
                    true,
                ));
            }
        }
    }

    // Sort generations because I don't know if the fs reports paths in any order
    generations.sort_by(|a, b| b.0.number.cmp(&a.0.number));

    let now = SystemTime::now();
    for gen in generations.iter_mut() {
        match now.duration_since(gen.0.last_modified) {
            Err(err) => {
                warn!(?err, ?now, ?gen, "Failed to compare time!");
            }
            Ok(val) if val <= keep_since.into() => {
                gen.1 = false;
            }
            Ok(_) => {}
        };
    }

    for gen in generations.iter_mut().take(keep as _) {
        gen.1 = false;
    }

    debug!("{:#?}", generations);
    Ok(generations)
}

#[derive(Debug)]
struct Generation {
    path: PathBuf,
    number: u32,
    last_modified: SystemTime,
}

fn prompt_clean(
    profiles: HashMap<PathBuf, Vec<(Generation, bool)>>,
    ask: bool,
    dry: bool,
) -> Result<()> {
    use owo_colors::OwoColorize;
    for (k, v) in profiles.iter() {
        println!("{}", k.to_string_lossy().bold().blue());
        for (gen, toberemoved) in v {
            if *toberemoved {
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

        for (_, v) in profiles.iter() {
            for (gen, toberemoved) in v {
                if *toberemoved {
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
