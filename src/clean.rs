use std::any;
use std::ffi::{CString, OsString};
use std::fs::{self, DirEntry, File};
use std::path::{Path, PathBuf};

use log::{info, trace};

use crate::commands::run_command;
use crate::{commands::NHRunnable, interface::CleanArgs};

// Reference: https://github.com/NixOS/nix/blob/master/src/nix-collect-garbage/nix-collect-garbage.cc

static PROFILES_DIR: &str = "/nix/var/nix/profiles";

impl NHRunnable for CleanArgs {
    fn run(&self) -> anyhow::Result<()> {
        let profiles_path = Path::new(PROFILES_DIR);

        // Clean profiles
        clean_profile(&profiles_path, self.dry)?;

        // Clean GC roots

        // Clean store
        run_command("nix-store --gc", self.dry, Some("Cleaning store"))?;
        Ok(())
    }
}

fn clean_profile(path: &Path, dry: bool) -> anyhow::Result<()> {
    for _dir_entry in fs::read_dir(path)? {
        let subpath = _dir_entry?.path();

        if !readable(&subpath)? {
            return Ok(());
        }

        let name = subpath.file_name().expect("FIXME").to_str().expect("FIXME");

        if name.contains("-link") && subpath.is_symlink() {
            // Is a generation
            let generation: Generation = subpath.into();

            if !generation.is_live() {
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

fn readable(path: &Path) -> Result<bool, anyhow::Error> {
    let fname = path.to_str().expect("FIXME");
    let cstr = CString::new(fname).expect("FIXME");
    let str_bytes = cstr.into_raw();
    Ok(unsafe { libc::access(str_bytes, libc::R_OK) } == 0)
}

#[derive(Debug)]
struct Generation {
    profile_name: String,
    number: u64,
    //
    path: PathBuf,
    profile_path: PathBuf,
    base_path: PathBuf,
}

impl From<PathBuf> for Generation {
    fn from(path: PathBuf) -> Self {
        let base_path: PathBuf = path
            .parent()
            .expect(&format!("Path {path:?} didn't have a parent!"))
            .into();

        // Something like (profile-name-with-dashes)-(number)-link
        let fname = path
            .file_name()
            .expect(&format!("Coudln't get filename for {path:?}"))
            .to_str()
            .unwrap();

        let mut fname_components: Vec<_> = fname.split("-").collect();

        // Remove link
        assert_eq!(fname_components.pop().unwrap(), "link");

        // Get number
        let number = fname_components.pop().unwrap().parse().unwrap();

        // The rest is the profile name
        let profile_name = fname_components.join("-");
        let profile_path = base_path.join(&profile_name);

        Generation {
            profile_name,
            number,
            path,
            profile_path,
            base_path,
        }
    }
}

impl Generation {
    fn is_live(&self) -> bool {
        let relative_pointing = fs::read_link(&self.profile_path).unwrap();
        let pointing = self.base_path.join(&relative_pointing);

        pointing == self.path
    }
}
