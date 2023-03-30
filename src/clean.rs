use std::ffi::CString;
use std::fs::{self};
use std::path::{Path, PathBuf};

use log::{info, trace, warn};

use crate::*;
use crate::{commands::NHRunnable, interface::CleanArgs};

// Reference: https://github.com/NixOS/nix/blob/master/src/nix-collect-garbage/nix-collect-garbage.cc

impl NHRunnable for CleanArgs {
    fn run(&self) -> anyhow::Result<()> {
        // FIXME
        // if !self.dry {
        //     crate::commands::check_root()?;
        // };

        // Clean profiles
        clean_profile(Path::new("/nix/var/nix/profiles"), self.dry)?;

        // Clean GC roots FIXME
        clean_gcroots(Path::new("/nix/var/nix/gcroots/auto"), self.dry)?;
        clean_gcroots(Path::new("/nix/var/nix/gcroots/per-user"), self.dry)?;

        // Clean store
        commands::CommandBuilder::default()
            .args(&["nix-store", "--gc"])
            .message("Calling nix-store --gc")
            .dry(self.dry)
            .build()?
            .run()?;

        Ok(())
    }
}

fn clean_profile(path: &Path, dry: bool) -> anyhow::Result<()> {
    for dir_entry in fs::read_dir(path)? {
        let subpath = dir_entry?.path();
        trace!("| subpath: {subpath:?}");

        if !readable(&subpath)? {
            return Ok(());
        }

        let name = subpath.file_name().expect("FIXME").to_str().expect("FIXME");

        if name.contains("-link") && subpath.is_symlink() {
            // Is a generation
            let generation: Generation = subpath.into();
            trace!("{generation:?}");

            let is_live = generation.is_live();
            trace!("live: {is_live}");

            if !is_live {
                remove_generation(&generation.path, dry)?;
            }
        } else if !subpath.is_symlink() && subpath.is_dir() {
            // Is a container for profiles
            clean_profile(&subpath, dry)?;
        }
    }

    Ok(())
}

fn remove_generation<P>(path: P, dry: bool) -> anyhow::Result<()>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    info!("Removing unused generation: {path:?}");
    if !dry {
        fs::remove_file(&path)?;
    }

    Ok(())
}

fn readable<P: AsRef<Path>>(path: P) -> nix::Result<bool> {
    let mode = nix::unistd::AccessFlags::F_OK;
    nix::unistd::eaccess(path.as_ref(), mode)?;
    Ok(true)
}

#[derive(Debug)]
struct Generation {
    // profile_name: String,
    // number: u64,
    path: PathBuf,
    profile_path: PathBuf,
    base_path: PathBuf,
}

impl From<PathBuf> for Generation {
    fn from(path: PathBuf) -> Self {
        let base_path: PathBuf = path
            .parent()
            .unwrap_or_else(|| panic!("Path {path:?} didn't have a parent!"))
            .into();

        // Something like (profile-name-with-dashes)-(number)-link
        let fname = path
            .file_name()
            .unwrap_or_else(|| panic!("Coudln't get filename for {path:?}"))
            .to_str()
            .unwrap();

        let mut fname_components: Vec<_> = fname.split('-').collect();

        // Remove link
        assert_eq!(fname_components.pop().unwrap(), "link");

        // Get number
        let _number = fname_components.pop();

        // The rest is the profile name
        let profile_name = fname_components.join("-");
        let profile_path = base_path.join(profile_name);

        Generation {
            // profile_name,
            // number,
            path,
            profile_path,
            base_path,
        }
    }
}

impl Generation {
    fn is_live(&self) -> bool {
        let relative_pointing = fs::read_link(&self.profile_path).unwrap();
        let pointing = self.base_path.join(relative_pointing);

        pointing == self.path
    }
}

fn clean_gcroots(path: &Path, dry: bool) -> anyhow::Result<()> {
    for dir_entry in fs::read_dir(path)? {
        let subpath = dir_entry?.path();
        trace!("| subpath: {subpath:?}");

        if !subpath.is_symlink() && subpath.is_dir() {
            // Walk inside
            clean_gcroots(&subpath, dry)?;
        } else if subpath.is_symlink() {
            let pointing = fs::read_link(&subpath)?;
            trace!("-> {pointing:?}");
            let pointed_fname = pointing
                .file_name()
                .expect("FIXME")
                .to_str()
                .expect("FIXME");

            if pointed_fname.contains("direnv") | pointed_fname.contains("result") {
                if pointing.try_exists()? {
                    info!("Removing GC root origin: {pointing:?}");
                    if !dry {
                        fs::remove_file(&pointing)?;
                    }
                } else {
                    warn!("GC root origin doesn't exist! {pointing:?}")
                }
            }
        }
    }

    Ok(())
}
