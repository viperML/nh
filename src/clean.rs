use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use color_eyre::eyre::{ensure, ContextCompat};
use color_eyre::owo_colors::OwoColorize;
use color_eyre::Result;
use derive_builder::Builder;
use log::{info, trace, warn};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::interface::NHRunnable;
use crate::interface::{CleanArgs, CleanMode, CleanProxy};

// Reference: https://github.com/NixOS/nix/blob/master/src/nix-collect-garbage/nix-collect-garbage.cc

impl NHRunnable for CleanMode {
    fn run(&self) -> Result<()> {
        match self {
            CleanMode::Info => todo!(),
            CleanMode::User(args) => clean_profiles(args),
            CleanMode::All(args) => clean_profiles(args),
        }
    }
}

fn clean_profiles(args: &CleanArgs) -> Result<()> {
    let base_dirs = [
        "/home/ayats/.local/state/nix/profiles",
        "/nix/var/nix/profiles",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect::<Vec<_>>();

    let mut profiles: HashMap<PathBuf, Vec<Generation>> = HashMap::new();

    for base_dir in base_dirs {
        let mut read = std::fs::read_dir(base_dir)?;

        while let Some(entry) = read.next() {
            // let x = x.await;
            let path = entry?.path();
            let parent = path.parent().unwrap();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();

            if let Some((base_name, id)) = parse_profile(&name) {
                let base_profile = parent.join(base_name);
                let last_modified: SystemTime = std::fs::symlink_metadata(&path)?.modified()?;

                let profile = Generation {
                    id,
                    path,
                    last_modified,
                    marked_for_deletion: true,
                };

                match profiles.get_mut(&base_profile) {
                    None => {
                        profiles.insert(base_profile, vec![profile]);
                    }
                    Some(v) => {
                        v.push(profile);
                    }
                };
            }

            if name.ends_with("-link") {}
        }
    }

    trace!("{:?}", profiles);
    info!("Calculating transaction");

    for (base_profile, mut generations) in profiles {
        let last_id = generations.last().unwrap().id;

        let base_profile_link = base_profile.read_link()?;
        let base_profile_link = base_profile_link.to_str().unwrap();
        let (_, base_profile_id) =
            parse_profile(base_profile_link).wrap_err("Parsing base profile")?;
        trace!("{:?}", base_profile_link);
        ensure!(
            base_profile_id == last_id,
            "Profile doesn't point into the generation with highest number, aborting"
        );

        eprintln!();
        eprintln!("- {}", base_profile.as_os_str().to_str().unwrap());

        for gen in generations.iter_mut() {
            // Use relative numbering, 1,2,3,4
            gen.id = last_id - gen.id + 1;

            if gen.id <= args.keep {
                gen.marked_for_deletion = false;
            }

            let age = SystemTime::now().duration_since(gen.last_modified)?;
            if age <= args.keep_since.into() {
                gen.marked_for_deletion = false;
            }

            if gen.marked_for_deletion {
                eprintln!("  🗑  {}", gen.path.to_str().unwrap());
            } else {
                eprintln!("  ✅ {}", gen.path.to_str().unwrap());
            }
        }

        trace!("{:?}", generations);
    }

    Ok(())
}


#[derive(Debug, Clone)]
struct Generation {
    id: u32,
    path: PathBuf,
    last_modified: SystemTime,
    marked_for_deletion: bool,
}

static PROFILE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.*)-(\d+)-link$").unwrap());

fn parse_profile<'s>(s: &'s str) -> Option<(&'s str, u32)> {
    let captures = PROFILE_PATTERN.captures(s)?;

    let base = captures.get(1)?.as_str();
    let number = captures.get(2)?.as_str().parse().ok()?;

    Some((base, number))
}

#[test]
fn test_parse_profile() {
    assert_eq!(
        parse_profile("home-manager-3-link"),
        Some(("home-manager", 3))
    );
    assert_eq!(
        parse_profile("home-manager-30-link"),
        Some(("home-manager", 30))
    );
    assert_eq!(parse_profile("home-manager"), None);
    assert_eq!(
        parse_profile("foo-bar-baz-0-link"),
        Some(("foo-bar-baz", 0))
    );
    assert_eq!(parse_profile("foo-bar-baz-X-link"), None);
}
