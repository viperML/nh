use std::fs::{self, DirEntry};
use std::path::PathBuf;

use log::trace;

use crate::commands::run_command;
use crate::{commands::NHRunnable, interface::CleanArgs};

// Reference: https://github.com/NixOS/nix/blob/master/src/nix-collect-garbage/nix-collect-garbage.cc

static PROFILES_DIR: &str = "/nix/var/nix/profiles";


impl NHRunnable for CleanArgs {
    fn run(&self) -> anyhow::Result<()> {
        let root = PathBuf::from(PROFILES_DIR);

        remove_old_generations(root)?;
        // run_command("nix-store --gc", self.dry, None)?;

        Ok(())
    }
}

fn remove_old_generations(path: PathBuf) -> anyhow::Result<()> {
    trace!("Working with path {path:?}");

    if path.is_dir() {
        trace!("Walking");
        for entry in fs::read_dir(path)? {
            let subpath = entry?.path();
            trace!("Going to {:?}", subpath);
            remove_old_generations(subpath)?;
        }

    } else {
        trace!("is file");
    }

    Ok(())
}
