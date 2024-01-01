use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::SystemTime,
};

use color_eyre::eyre::{Context, ContextCompat};
use once_cell::sync::Lazy;
use regex::Regex;
use tracing::{debug, instrument, trace, warn};

use crate::*;

// Reference: https://github.com/NixOS/nix/blob/master/src/nix-collect-garbage/nix-collect-garbage.cc

impl NHRunnable for interface::CleanMode {
    fn run(&self) -> Result<()> {
        let uid = nix::unistd::Uid::effective();

        match self {
            interface::CleanMode::Profile(args) => {
                // cleanable_generations(args., keep, keep_size)
                cleanable_generations(&args.profile, args.common.keep, args.common.keep_since)?;
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

// static PROFILE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.*)-(\d+)-link$").unwrap());

// fn parse_profile(s: &str) -> Option<(&str, u32)> {
//     let captures = PROFILE_PATTERN.captures(s)?;

//     let base = captures.get(1)?.as_str();
//     let number = captures.get(2)?.as_str().parse().ok()?;

//     Some((base, number))
// }

// #[test]
// fn test_parse_profile() {
//     assert_eq!(
//         parse_profile("home-manager-3-link"),
//         Some(("home-manager", 3))
//     );
//     assert_eq!(
//         parse_profile("home-manager-30-link"),
//         Some(("home-manager", 30))
//     );
//     assert_eq!(parse_profile("home-manager"), None);
//     assert_eq!(
//         parse_profile("foo-bar-baz-0-link"),
//         Some(("foo-bar-baz", 0))
//     );
//     assert_eq!(parse_profile("foo-bar-baz-X-link"), None);
// }
