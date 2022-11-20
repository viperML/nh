use std::ffi::{CString, OsString};
use std::fs::{self, DirEntry, File};
use std::path::{PathBuf, Path};

use log::trace;

use crate::commands::run_command;
use crate::{commands::NHRunnable, interface::CleanArgs};

// Reference: https://github.com/NixOS/nix/blob/master/src/nix-collect-garbage/nix-collect-garbage.cc

static PROFILES_DIR: &str = "/nix/var/nix/profiles";

impl NHRunnable for CleanArgs {
    fn run(&self) -> anyhow::Result<()> {
        let profiles_path = Path::new(PROFILES_DIR);

        // Clean profiles
        clean_profile(&profiles_path)?;

        // Clean GC roots

        // Clean store
        // run_command("nix-store --gc", self.dry, None)?;
        Ok(())
    }
}

fn clean_profile(path: &Path) -> anyhow::Result<()> {
    for _dir_entry in fs::read_dir(path)? {
        let subpath = _dir_entry?.path();

        if !readable(&subpath)? {
            return Ok(());
        }

        let name = subpath.file_name().expect("FIXME").to_str().expect("FIXME");

        if name.contains("-link") && subpath.is_symlink() {
            // Is a profile
            trace!("Wipe-checking {subpath:?}");
            // Clean profile
        } else if !subpath.is_symlink() && subpath.is_dir() {
            // Is a container for profiles
            clean_profile(&subpath)?;
        }
    }

    Ok(())
}

fn readable(path: &Path) -> Result<bool, anyhow::Error> {
    let fname = path.to_str().expect("FIXME");
    let cstr = CString::new(fname).expect("FIXME");
    let str_bytes = cstr.into_raw();
    Ok(unsafe { libc::access(str_bytes, libc::R_OK) } == 0)
}
