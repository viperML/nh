use std::collections::HashMap;
use std::os::unix::process::CommandExt;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use color_eyre::eyre::{bail, ensure, ContextCompat};
use color_eyre::Result;
use log::{debug, info, trace, warn};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::commands;
use crate::interface::NHRunnable;
use crate::interface::{CleanArgs, CleanMode};

// Reference: https://github.com/NixOS/nix/blob/master/src/nix-collect-garbage/nix-collect-garbage.cc

impl NHRunnable for CleanMode {
    fn run(&self) -> Result<()> {
        match self {
            CleanMode::Info => todo!(),
            CleanMode::All(args) => {
                let uid = nix::unistd::Uid::effective();
                if !uid.is_root() {
                    let mut cmd = std::process::Command::new("sudo");
                    cmd.args(std::env::args());
                    debug!("{:?}", cmd);
                    let err = cmd.exec();
                    bail!(err);
                }

                let mut profiles = Vec::new();
                let mut gcroots = Vec::new();

                gcroots.push(PathBuf::from("/nix/var/nix/gcroots/auto"));
                profiles.push(PathBuf::from("/nix/var/nix/profiles"));

                for entry in std::fs::read_dir("/home")? {
                    let homedir = entry?.path();
                    profiles.push(homedir.join(".local/state/nix/profiles"));
                }
                for entry in std::fs::read_dir("/nix/var/nix/profiles/per-user")? {
                    let path = entry?.path();
                    profiles.push(path);
                }
                for entry in std::fs::read_dir("/nix/var/nix/gcroots/per-user")? {
                    let path = entry?.path();
                    gcroots.push(path);
                }

                clean(args, &profiles, &gcroots)
            }
            CleanMode::User(args) => {
                let uid = nix::unistd::Uid::effective();
                if uid.is_root() {
                    bail!("nh clean user: don't run me as root!");
                }
                let user = nix::unistd::User::from_uid(uid)?.unwrap();
                let home = PathBuf::from(std::env::var("HOME")?);
                clean(
                    args,
                    &[
                        PathBuf::from("/nix/var/nix/profiles/per-user").join(&user.name),
                        home.join(".local/state/nix/profiles"),
                    ],
                    &[PathBuf::from("/nix/var/nix/gcroots/per-user").join(&user.name)],
                )
            }
        }
    }
}

fn clean<P>(args: &CleanArgs, base_dirs: &[P], gcroots_dirs: &[P]) -> Result<()>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    info!("Calculating transaction");
    trace!("{:?}", gcroots_dirs);

    let mut gc_roots_to_remove = Vec::new();
    if !args.nogcroots {
        for dir in gcroots_dirs {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?.path();
                trace!("Checking entry {:?}", entry);

                let pointing_to = match std::fs::read_link(&entry) {
                    Ok(p) => p,
                    Err(err) => match err.kind() {
                        std::io::ErrorKind::NotFound => continue,
                        other => bail!(other),
                    },
                };

                let last_modified = std::fs::symlink_metadata(&entry)?.modified()?;
                if SystemTime::now().duration_since(last_modified)? <= args.keep_since.into() {
                    continue;
                }

                let delete = pointing_to.components().any(|comp| {
                    if let Component::Normal(s) = comp {
                        let s = s.to_str().expect("Couldn't convert OsStr to UTF-8 str");
                        if s == ".direnv" {
                            return true;
                        }
                        if s.contains("result") {
                            return true;
                        }
                    };
                    return false;
                });

                if delete {
                    eprintln!(
                        " 🗑  {} -> {}",
                        entry.to_str().unwrap(),
                        pointing_to.to_str().unwrap()
                    );
                    gc_roots_to_remove.push(entry);
                };
            }
        }
    }

    trace!("{:?}", base_dirs);
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

    for (base_profile, generations) in &mut profiles {
        let last_id = generations.last().unwrap().id;

        let base_profile_link = base_profile.read_link()?;
        let base_profile_link = base_profile_link.to_str().unwrap();
        let (_, base_profile_id) =
            parse_profile(base_profile_link).wrap_err("Parsing base profile")?;
        trace!("({base_profile_id:?}) {}", base_profile_link);
        trace!(
            "({last_id:?}) {}",
            generations.last().unwrap().path.to_str().unwrap()
        );
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

    if args.dry {
        return Ok(());
    }

    if args.ask {
        info!("Confirm the cleanup plan?");
        let confirmation = dialoguer::Confirm::new().default(false).interact()?;
        if !confirmation {
            return Ok(());
        }
    }

    for root in gc_roots_to_remove {
        info!("Removing {:?}", root);
        if let Err(e) = std::fs::remove_file(root) {
            warn!("Failed to remove: {:?}", e);
        }
    }

    for (_, generations) in profiles {
        for gen in generations {
            if gen.marked_for_deletion {
                info!("Removing {:?}", gen.path);
                if let Err(e) = std::fs::remove_file(gen.path) {
                    warn!("Failed to remove: {:?}", e);
                }
            }
        }
    }

    if !args.nogc {
        commands::CommandBuilder::default()
            .args(&["nix", "store", "gc"])
            .message("nix store gc")
            .build()?
            .exec()?;
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
